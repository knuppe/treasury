//! Regras de categorização automática (aplicadas na importação e sob demanda).
//!
//! - Texto compara sem diferenciar maiúsculas e acentos; `Regex` usa o crate `regex`
//!   (tempo linear, sem retrocesso exponencial), compilada uma vez por regra.
//! - Prioridade: menor primeiro. A primeira regra que define um campo vence.
//! - Descrições, favorecidos e notas são **dados**: as regras só comparam texto, nunca
//!   interpretam o conteúdo como comando.
//! - Os acertos (`hits`) ficam na tabela técnica `rule_hit`, fora do histórico, para que
//!   contadores não impeçam desfazer importações (ver V20).

pub mod api;
pub mod builtin;
pub mod learned;
pub(crate) mod views;

use std::collections::HashMap;

use regex::{Regex, RegexBuilder};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::import::text::fold;
use crate::time::now_utc;
use crate::transactions::{NewTxn, TxnKind, TxnView, fold_accent};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RuleField {
    Description,
    Payee,
    Notes,
    /// Valor com sinal, em centavos.
    Amount,
    AccountId,
    Kind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RuleOp {
    /// Texto: contém (sem diferenciar maiúsculas e acentos).
    Contains,
    Equals,
    StartsWith,
    EndsWith,
    /// Expressão regular (sintaxe do crate `regex`).
    Regex,
    /// Número: entre `value` e `value2` (inclusive).
    Between,
    Gte,
    Lte,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
pub struct RuleCondition {
    pub field: RuleField,
    pub op: RuleOp,
    /// Texto ou número, conforme o campo.
    #[ts(type = "string | number")]
    pub value: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "number")]
    pub value2: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
pub struct RuleActions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub category_id: Option<Id>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub member_id: Option<Id>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tags: Option<Vec<String>>,
    /// Renomear a descrição.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub rename_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub kind: Option<TxnKind>,
}

impl RuleActions {
    pub fn is_empty(&self) -> bool {
        self.category_id.is_none()
            && self.member_id.is_none()
            && self.tags.as_ref().is_none_or(|t| t.is_empty())
            && self.rename_to.is_none()
            && self.kind.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Rule {
    pub id: Id,
    pub name: String,
    /// Menor primeiro. A primeira regra que casa define cada campo.
    #[ts(type = "number")]
    pub priority: i64,
    pub active: bool,
    /// Todas as condições (E) ou qualquer uma (OU).
    pub match_all: bool,
    pub conditions: Vec<RuleCondition>,
    pub actions: RuleActions,
    #[ts(type = "number")]
    pub hits: i64,
    /// Último lançamento categorizado pela regra (UTC).
    pub last_hit_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RuleInput {
    pub name: String,
    #[ts(optional, type = "number")]
    pub priority: Option<i64>,
    #[ts(optional)]
    pub active: Option<bool>,
    #[ts(optional)]
    pub match_all: Option<bool>,
    pub conditions: Vec<RuleCondition>,
    pub actions: RuleActions,
}

/// `rules.update`: substitui a regra inteira.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RuleUpdate {
    pub id: Id,
    #[serde(flatten)]
    pub input: RuleInput,
}

/// `rules.test`: simula uma regra (ainda não salva) contra os lançamentos existentes.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RuleTestArgs {
    pub rule: RuleInput,
    /// Máximo de lançamentos devolvidos (padrão 50, até 500).
    #[ts(optional, type = "number")]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RuleApplyArgs {
    /// Regras a aplicar (padrão: todas as ativas).
    #[ts(optional)]
    pub rule_ids: Option<Vec<Id>>,
    /// Lançamentos alvo (padrão: todos).
    #[ts(optional)]
    pub filter: Option<crate::transactions::TxnFilter>,
    /// Só lançamentos sem categoria (padrão: sim).
    #[ts(optional)]
    pub only_uncategorized: Option<bool>,
    /// Só simular, sem gravar.
    #[ts(optional)]
    pub dry_run: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RuleApplyResult {
    #[ts(type = "number")]
    pub matched: i64,
    #[ts(type = "number")]
    pub updated: i64,
    /// Amostra dos lançamentos afetados (até 50).
    pub sample: Vec<crate::transactions::TxnView>,
    pub change_set_id: Option<Id>,
}

// ---------------------------------------------------------------------------
// Motor

#[derive(Debug, Clone)]
enum Matcher {
    Text {
        field: RuleField,
        op: RuleOp,
        needle: String,
    },
    Regex {
        field: RuleField,
        re: Regex,
    },
    Amount {
        op: RuleOp,
        a: i64,
        b: i64,
    },
    Account(Id),
    Kind(TxnKind),
}

/// Regra pronta para avaliar (regex já compilada).
#[derive(Debug, Clone)]
pub struct CompiledRule {
    pub id: Id,
    pub name: String,
    pub priority: i64,
    match_all: bool,
    matchers: Vec<Matcher>,
    pub actions: RuleActions,
}

/// Campos de um lançamento que as regras examinam.
#[derive(Debug, Clone, Copy)]
pub struct Subject<'a> {
    pub description: &'a str,
    pub payee: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub amount: i64,
    pub account_id: &'a str,
    pub kind: TxnKind,
}

impl<'a> From<&'a NewTxn> for Subject<'a> {
    fn from(t: &'a NewTxn) -> Self {
        Subject {
            description: &t.description,
            payee: t.payee.as_deref(),
            notes: t.notes.as_deref(),
            amount: t.amount,
            account_id: &t.account_id,
            kind: t.kind,
        }
    }
}

/// O que as regras definem para um lançamento (cada campo vem da primeira regra que o
/// define, em ordem de prioridade).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RuleOutcome {
    pub category_id: Option<Id>,
    pub member_id: Option<Id>,
    pub tags: Option<Vec<String>>,
    pub rename_to: Option<String>,
    pub kind: Option<TxnKind>,
    /// Regras que casaram, em ordem de prioridade.
    pub matched: Vec<Id>,
    /// Regras que definiram pelo menos um campo (contam como acerto).
    pub applied: Vec<Id>,
}

impl RuleOutcome {
    pub fn first_rule(&self) -> Option<&Id> {
        self.applied.first().or(self.matched.first())
    }
}

fn strip_accents(s: &str) -> String {
    s.chars().map(fold_accent).collect()
}

fn text_value(c: &RuleCondition) -> Result<String> {
    match &c.value {
        serde_json::Value::String(s) if !s.trim().is_empty() => Ok(s.clone()),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        _ => Err(Error::invalid(t("nucleo-lancamentos-regra-erro-texto"))),
    }
}

fn amount_value(v: &serde_json::Value) -> Result<i64> {
    match v {
        serde_json::Value::Number(n) => n.as_i64().ok_or_else(|| {
            Error::invalid(t_args(
                "nucleo-lancamentos-regra-erro-centavos",
                // A regra vale para contas de qualquer moeda: o exemplo vai sem símbolo.
                &args(&[(
                    "exemplo",
                    crate::i18n::number(crate::i18n::current(), -5000).into(),
                )]),
            ))
        }),
        // Só as casas decimais contam aqui (2, como na maioria das moedas).
        serde_json::Value::String(s) => crate::money::parse_amount(s, "BRL"),
        _ => Err(Error::invalid(t("nucleo-lancamentos-regra-erro-numero"))),
    }
}

fn compile_condition(c: &RuleCondition) -> Result<Matcher> {
    use RuleField as F;
    use RuleOp as O;
    match c.field {
        F::Description | F::Payee | F::Notes => match c.op {
            O::Regex => {
                let pat = text_value(c)?;
                let re = RegexBuilder::new(&strip_accents(&pat))
                    .case_insensitive(true)
                    .size_limit(1 << 20)
                    .dfa_size_limit(1 << 22)
                    .build()
                    .map_err(|e| {
                        Error::invalid(t_args(
                            "nucleo-lancamentos-regra-erro-regex",
                            &args(&[("erro", e.to_string().into())]),
                        ))
                    })?;
                Ok(Matcher::Regex { field: c.field, re })
            }
            O::Contains | O::Equals | O::StartsWith | O::EndsWith => {
                let needle = fold(&text_value(c)?);
                if needle.is_empty() {
                    return Err(Error::invalid(t(
                        "nucleo-lancamentos-regra-erro-texto-vazio",
                    )));
                }
                Ok(Matcher::Text {
                    field: c.field,
                    op: c.op,
                    needle,
                })
            }
            _ => Err(Error::invalid(t("nucleo-lancamentos-regra-erro-op-texto"))),
        },
        F::Amount => {
            let a = amount_value(&c.value)?;
            match c.op {
                O::Equals | O::Gte | O::Lte => Ok(Matcher::Amount { op: c.op, a, b: a }),
                O::Between => {
                    let b = c
                        .value2
                        .ok_or_else(|| Error::invalid(t("nucleo-lancamentos-regra-erro-entre")))?;
                    Ok(Matcher::Amount {
                        op: c.op,
                        a: a.min(b),
                        b: a.max(b),
                    })
                }
                _ => Err(Error::invalid(t("nucleo-lancamentos-regra-erro-op-valor"))),
            }
        }
        F::AccountId => match (c.op, &c.value) {
            (O::Equals, serde_json::Value::String(s)) if !s.is_empty() => {
                Ok(Matcher::Account(s.clone()))
            }
            _ => Err(Error::invalid(t("nucleo-lancamentos-regra-erro-op-conta"))),
        },
        F::Kind => match (c.op, &c.value) {
            (O::Equals, serde_json::Value::String(s)) => Ok(Matcher::Kind(TxnKind::parse(s)?)),
            _ => Err(Error::invalid(t("nucleo-lancamentos-regra-erro-op-tipo"))),
        },
    }
}

impl Matcher {
    fn matches(&self, s: &Subject<'_>) -> bool {
        let field_text = |f: RuleField| -> &str {
            match f {
                RuleField::Description => s.description,
                RuleField::Payee => s.payee.unwrap_or(""),
                RuleField::Notes => s.notes.unwrap_or(""),
                _ => "",
            }
        };
        match self {
            Matcher::Text { field, op, needle } => {
                let hay = fold(field_text(*field));
                match op {
                    RuleOp::Contains => hay.contains(needle.as_str()),
                    RuleOp::Equals => hay == *needle,
                    RuleOp::StartsWith => hay.starts_with(needle.as_str()),
                    RuleOp::EndsWith => hay.ends_with(needle.as_str()),
                    _ => false,
                }
            }
            Matcher::Regex { field, re } => re.is_match(&strip_accents(field_text(*field))),
            Matcher::Amount { op, a, b } => match op {
                RuleOp::Equals => s.amount == *a,
                RuleOp::Gte => s.amount >= *a,
                RuleOp::Lte => s.amount <= *a,
                RuleOp::Between => (*a..=*b).contains(&s.amount),
                _ => false,
            },
            Matcher::Account(id) => s.account_id == id,
            Matcher::Kind(k) => s.kind == *k,
        }
    }
}

impl CompiledRule {
    pub fn compile(id: &str, name: &str, priority: i64, input: &RuleInput) -> Result<Self> {
        if input.conditions.is_empty() {
            return Err(Error::invalid(t(
                "nucleo-lancamentos-regra-erro-sem-condicao",
            )));
        }
        let matchers = input
            .conditions
            .iter()
            .map(compile_condition)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            id: id.to_string(),
            name: name.to_string(),
            priority,
            match_all: input.match_all.unwrap_or(true),
            matchers,
            actions: input.actions.clone(),
        })
    }

    pub fn matches(&self, s: &Subject<'_>) -> bool {
        if self.matchers.is_empty() {
            return false;
        }
        if self.match_all {
            self.matchers.iter().all(|m| m.matches(s))
        } else {
            self.matchers.iter().any(|m| m.matches(s))
        }
    }
}

/// Conjunto de regras ordenado por prioridade.
#[derive(Debug, Clone, Default)]
pub struct RuleSet {
    rules: Vec<CompiledRule>,
}

impl RuleSet {
    pub fn new(mut rules: Vec<CompiledRule>) -> Self {
        rules.sort_by_key(|a| a.priority);
        Self { rules }
    }

    /// Todas as regras ativas.
    pub fn load_active(conn: &Connection) -> Result<Self> {
        Self::load(conn, None)
    }

    /// Regras escolhidas (mesmo inativas) ou, sem `ids`, todas as ativas.
    pub fn load(conn: &Connection, ids: Option<&[Id]>) -> Result<Self> {
        let rules = list(conn)?;
        let mut compiled = Vec::new();
        for r in rules {
            let wanted = match ids {
                Some(ids) => ids.contains(&r.id),
                None => r.active,
            };
            if !wanted {
                continue;
            }
            let input = RuleInput {
                name: r.name.clone(),
                priority: Some(r.priority),
                active: Some(r.active),
                match_all: Some(r.match_all),
                conditions: r.conditions.clone(),
                actions: r.actions.clone(),
            };
            // Uma regra gravada que não compila mais (ex.: editada por fora) é ignorada.
            if let Ok(c) = CompiledRule::compile(&r.id, &r.name, r.priority, &input) {
                compiled.push(c);
            }
        }
        if let Some(ids) = ids
            && let Some(missing) = ids.iter().find(|id| !compiled.iter().any(|c| &c.id == *id))
        {
            return Err(Error::not_found(rule_ref(missing)));
        }
        Ok(Self::new(compiled))
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn rules(&self) -> &[CompiledRule] {
        &self.rules
    }

    pub fn evaluate(&self, s: &Subject<'_>) -> RuleOutcome {
        let mut out = RuleOutcome::default();
        for r in &self.rules {
            if !r.matches(s) {
                continue;
            }
            out.matched.push(r.id.clone());
            let a = &r.actions;
            let mut contributed = false;
            if out.category_id.is_none() && a.category_id.is_some() {
                out.category_id = a.category_id.clone();
                contributed = true;
            }
            if out.member_id.is_none() && a.member_id.is_some() {
                out.member_id = a.member_id.clone();
                contributed = true;
            }
            if out.tags.is_none() && a.tags.as_ref().is_some_and(|t| !t.is_empty()) {
                out.tags = a.tags.clone();
                contributed = true;
            }
            if out.rename_to.is_none() && a.rename_to.as_ref().is_some_and(|t| !t.trim().is_empty())
            {
                out.rename_to = a.rename_to.as_ref().map(|s| s.trim().to_string());
                contributed = true;
            }
            if out.kind.is_none() && a.kind.is_some() {
                out.kind = a.kind;
                contributed = true;
            }
            if contributed {
                out.applied.push(r.id.clone());
            }
        }
        out
    }

    /// Aplica as regras a um lançamento ainda não gravado: preenche categoria e titular
    /// vazios, renomeia e troca o tipo quando a regra manda. Categoria não é aplicada a
    /// pagamentos de fatura, aportes e ajustes. As tags ficam em `RuleOutcome::tags`
    /// (grave com `transactions::replace_tags` depois do `insert`).
    pub fn apply_to_new(&self, t: &mut NewTxn) -> RuleOutcome {
        let out = self.evaluate(&Subject::from(&*t));
        if let Some(k) = out.kind {
            t.kind = k;
        }
        if t.category_id.is_none() && takes_category(t.kind) {
            t.category_id = out.category_id.clone();
        }
        if t.member_id.is_none() {
            t.member_id = out.member_id.clone();
        }
        if let Some(name) = &out.rename_to {
            t.description = name.clone();
        }
        out
    }
}

/// Atalho para quem tem um único lançamento (ex.: agente de IA): carrega as regras ativas
/// e aplica. Para muitos lançamentos, carregue um [`RuleSet`] uma vez.
pub fn apply_to_new(conn: &Connection, t: &mut NewTxn) -> Result<RuleOutcome> {
    Ok(RuleSet::load_active(conn)?.apply_to_new(t))
}

/// Soma acertos (tabela técnica, fora do histórico).
pub fn record_hits(conn: &Connection, hits: &HashMap<Id, i64>) -> Result<()> {
    let now = now_utc();
    let mut stmt = conn.prepare_cached(
        "INSERT INTO rule_hit (rule_id, hits, last_hit_at) VALUES (?1, ?2, ?3)
         ON CONFLICT (rule_id) DO UPDATE SET hits = hits + excluded.hits, last_hit_at = excluded.last_hit_at",
    )?;
    for (id, n) in hits.iter().filter(|(_, n)| **n > 0) {
        stmt.execute(params![id, n, now])?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Persistência

#[derive(Serialize, Deserialize)]
struct StoredConditions {
    #[serde(default = "yes")]
    match_all: bool,
    items: Vec<RuleCondition>,
}

fn yes() -> bool {
    true
}

fn parse_conditions(raw: &str) -> (bool, Vec<RuleCondition>) {
    if let Ok(s) = serde_json::from_str::<StoredConditions>(raw) {
        return (s.match_all, s.items);
    }
    // Tolerância: lista pura de condições.
    match serde_json::from_str::<Vec<RuleCondition>>(raw) {
        Ok(items) => (true, items),
        Err(_) => (true, vec![]),
    }
}

const RULE_COLS: &str = "r.id, r.name, r.priority, r.active, r.conditions, r.actions,
     r.hits + COALESCE(h.hits, 0), h.last_hit_at, r.created_at, r.updated_at";

fn row_to_rule(r: &rusqlite::Row<'_>) -> rusqlite::Result<Rule> {
    let conditions: String = r.get(4)?;
    let actions: String = r.get(5)?;
    let (match_all, conditions) = parse_conditions(&conditions);
    Ok(Rule {
        id: r.get(0)?,
        name: r.get(1)?,
        priority: r.get(2)?,
        active: r.get::<_, i64>(3)? != 0,
        match_all,
        conditions,
        actions: serde_json::from_str(&actions).unwrap_or_default(),
        hits: r.get(6)?,
        last_hit_at: r.get(7)?,
        created_at: r.get(8)?,
        updated_at: r.get(9)?,
    })
}

pub fn list(conn: &Connection) -> Result<Vec<Rule>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {RULE_COLS} FROM rule r LEFT JOIN rule_hit h ON h.rule_id = r.id
         ORDER BY r.priority, r.created_at, r.id"
    ))?;
    let rows = stmt
        .query_map([], row_to_rule)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, id: &str) -> Result<Rule> {
    conn.query_row(
        &format!("SELECT {RULE_COLS} FROM rule r LEFT JOIN rule_hit h ON h.rule_id = r.id WHERE r.id = ?1"),
        [id],
        row_to_rule,
    )
    .optional()?
    .ok_or_else(|| Error::not_found(rule_ref(id)))
}

/// "regra {id}" (detalhe do erro "não encontrado").
fn rule_ref(id: &str) -> String {
    t_args(
        "nucleo-lancamentos-ref-regra",
        &args(&[("id", id.to_string().into())]),
    )
}

fn exists(conn: &Connection, table: &str, id: &str) -> Result<bool> {
    Ok(conn.query_row(
        &format!("SELECT EXISTS (SELECT 1 FROM {table} WHERE id = ?1)"),
        [id],
        |r| r.get(0),
    )?)
}

/// Valida a regra: condições compiláveis, ações não vazias, referências existentes.
pub fn validate(conn: &Connection, input: &RuleInput) -> Result<()> {
    if input.name.trim().is_empty() {
        return Err(Error::invalid(t("nucleo-lancamentos-regra-erro-nome")));
    }
    CompiledRule::compile("", &input.name, 0, input)?;
    let a = &input.actions;
    if a.is_empty() {
        return Err(Error::invalid(t("nucleo-lancamentos-regra-erro-sem-acao")));
    }
    if let Some(c) = &a.category_id
        && !exists(conn, "category", c)?
    {
        return Err(Error::not_found(crate::categories::category_ref(c)));
    }
    if let Some(m) = &a.member_id
        && !exists(conn, "member", m)?
    {
        return Err(Error::not_found(crate::transactions::util::member_ref(m)));
    }
    if a.rename_to.as_ref().is_some_and(|r| r.trim().is_empty()) {
        return Err(Error::invalid(t("nucleo-lancamentos-regra-erro-renomear")));
    }
    if a.tags
        .as_ref()
        .is_some_and(|t| t.iter().any(|t| t.trim().is_empty()))
    {
        return Err(Error::invalid(t("nucleo-lancamentos-regra-erro-tags")));
    }
    for c in &input.conditions {
        if c.field == RuleField::AccountId
            && let serde_json::Value::String(id) = &c.value
            && !exists(conn, "account", id)?
        {
            return Err(Error::not_found(crate::accounts::account_ref(id)));
        }
    }
    Ok(())
}

fn conditions_json(input: &RuleInput) -> Result<String> {
    Ok(serde_json::to_string(&StoredConditions {
        match_all: input.match_all.unwrap_or(true),
        items: input.conditions.clone(),
    })?)
}

fn clean_actions(a: &RuleActions) -> RuleActions {
    RuleActions {
        tags: a.tags.as_ref().map(|t| {
            let mut out: Vec<String> = Vec::new();
            for t in t.iter().map(|t| t.trim().to_string()) {
                if !out.iter().any(|o| o.eq_ignore_ascii_case(&t)) {
                    out.push(t);
                }
            }
            out
        }),
        rename_to: a.rename_to.as_ref().map(|r| r.trim().to_string()),
        ..a.clone()
    }
}

pub fn create(tx: &Transaction<'_>, input: &RuleInput) -> Result<Id> {
    validate(tx, input)?;
    let id = new_id();
    let now = now_utc();
    tx.execute(
        "INSERT INTO rule (id, name, priority, active, conditions, actions, hits, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?7)",
        params![
            id,
            input.name.trim(),
            input.priority.unwrap_or(100),
            input.active.unwrap_or(true) as i64,
            conditions_json(input)?,
            serde_json::to_string(&clean_actions(&input.actions))?,
            now
        ],
    )?;
    Ok(id)
}

pub fn update(tx: &Transaction<'_>, id: &str, input: &RuleInput) -> Result<()> {
    let current = get(tx, id)?;
    validate(tx, input)?;
    tx.execute(
        "UPDATE rule SET name = ?2, priority = ?3, active = ?4, conditions = ?5, actions = ?6, updated_at = ?7
         WHERE id = ?1",
        params![
            id,
            input.name.trim(),
            input.priority.unwrap_or(current.priority),
            input.active.unwrap_or(current.active) as i64,
            conditions_json(input)?,
            serde_json::to_string(&clean_actions(&input.actions))?,
            now_utc()
        ],
    )?;
    Ok(())
}

pub fn delete(tx: &Transaction<'_>, id: &str) -> Result<()> {
    if tx.execute("DELETE FROM rule WHERE id = ?1", [id])? == 0 {
        return Err(Error::not_found(rule_ref(id)));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Simulação, aplicação e sugestão

struct TxnFacts {
    id: Id,
    description: String,
    payee: Option<String>,
    notes: Option<String>,
    amount: i64,
    account_id: Id,
    kind: TxnKind,
    category_id: Option<Id>,
    member_id: Option<Id>,
    has_splits: bool,
    /// Perna de transferência entre contas ou lançamento vinculado a uma operação: o tipo
    /// segue o vínculo, não a regra.
    linked: bool,
}

impl TxnFacts {
    fn subject(&self) -> Subject<'_> {
        Subject {
            description: &self.description,
            payee: self.payee.as_deref(),
            notes: self.notes.as_deref(),
            amount: self.amount,
            account_id: &self.account_id,
            kind: self.kind,
        }
    }
}

const FACT_COLS: &str = "t.id, t.description, t.payee, t.notes, t.amount, t.account_id, t.kind,
     t.category_id, t.member_id, EXISTS (SELECT 1 FROM txn_split s WHERE s.txn_id = t.id),
     t.transfer_id IS NOT NULL OR EXISTS (SELECT 1 FROM txn_link l WHERE l.txn_id = t.id)";

fn row_to_facts(r: &rusqlite::Row<'_>) -> rusqlite::Result<TxnFacts> {
    let kind: String = r.get(6)?;
    Ok(TxnFacts {
        id: r.get(0)?,
        description: r.get(1)?,
        payee: r.get(2)?,
        notes: r.get(3)?,
        amount: r.get(4)?,
        account_id: r.get(5)?,
        kind: TxnKind::parse(&kind).unwrap_or(TxnKind::Adjustment),
        category_id: r.get(7)?,
        member_id: r.get(8)?,
        has_splits: r.get(9)?,
        linked: r.get(10)?,
    })
}

fn facts_for(conn: &Connection, ids: Option<&[Id]>) -> Result<Vec<TxnFacts>> {
    match ids {
        None => {
            let mut stmt = conn.prepare(&format!(
                "SELECT {FACT_COLS} FROM txn t ORDER BY t.date DESC, t.created_at DESC, t.id DESC"
            ))?;
            let rows = stmt
                .query_map([], row_to_facts)?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(rows)
        }
        Some(ids) => {
            let mut stmt =
                conn.prepare_cached(&format!("SELECT {FACT_COLS} FROM txn t WHERE t.id = ?1"))?;
            let mut out = Vec::with_capacity(ids.len());
            for id in ids {
                if let Some(f) = stmt.query_row([id], row_to_facts).optional()? {
                    out.push(f);
                }
            }
            Ok(out)
        }
    }
}

/// Lançamentos existentes que a regra pegaria (mais recentes primeiro), sem gravar nada.
pub fn test(conn: &Connection, args: &RuleTestArgs) -> Result<Vec<TxnView>> {
    let rule = CompiledRule::compile("teste", &args.rule.name, 0, &args.rule)?;
    let limit = args.limit.unwrap_or(50).clamp(1, 500) as usize;
    let mut ids = Vec::new();
    for f in facts_for(conn, None)? {
        if rule.matches(&f.subject()) {
            ids.push(f.id);
            if ids.len() >= limit {
                break;
            }
        }
    }
    views::load(conn, &ids)
}

/// Tipos que levam categoria: receita, despesa e transferência (a de uma perna só conta
/// no fluxo pela categoria). Fatura, aporte e ajuste não.
fn takes_category(k: TxnKind) -> bool {
    matches!(k, TxnKind::Income | TxnKind::Expense | TxnKind::Transfer)
}

#[derive(Debug, Clone)]
struct Planned {
    id: Id,
    category_id: Option<Option<Id>>,
    member_id: Option<Id>,
    description: Option<String>,
    kind: Option<TxnKind>,
    add_tags: Vec<String>,
    rules: Vec<Id>,
}

struct Plan {
    matched: i64,
    changes: Vec<Planned>,
}

fn plan(conn: &Connection, args: &RuleApplyArgs) -> Result<Plan> {
    let set = RuleSet::load(conn, args.rule_ids.as_deref())?;
    let only_uncategorized = args.only_uncategorized.unwrap_or(true);
    let facts = match &args.filter {
        Some(f) => {
            let ids = views::filter_ids(conn, f)?;
            facts_for(conn, Some(&ids))?
        }
        None => facts_for(conn, None)?,
    };
    let mut existing_tags = conn.prepare_cached(
        "SELECT g.name FROM txn_tag tt JOIN tag g ON g.id = tt.tag_id WHERE tt.txn_id = ?1",
    )?;
    let mut matched = 0;
    let mut changes = Vec::new();
    for f in facts {
        if only_uncategorized && (f.category_id.is_some() || f.has_splits) {
            continue;
        }
        let out = set.evaluate(&f.subject());
        if out.matched.is_empty() {
            continue;
        }
        matched += 1;
        let mut p = Planned {
            id: f.id.clone(),
            category_id: None,
            member_id: None,
            description: None,
            kind: None,
            add_tags: vec![],
            rules: out.applied.clone(),
        };
        // Pernas ligadas mantêm o tipo; lançamento dividido não vira transferência.
        let rule_kind = out.kind.filter(|k| {
            !f.linked && !(f.has_splits && !matches!(k, TxnKind::Income | TxnKind::Expense))
        });
        let final_kind = rule_kind.unwrap_or(f.kind);
        if rule_kind.is_some_and(|k| k != f.kind) {
            p.kind = rule_kind;
        }
        let may_override = f.category_id.is_none() || !only_uncategorized;
        if !takes_category(final_kind) {
            if f.category_id.is_some() && p.kind.is_some() {
                p.category_id = Some(None);
            }
        } else if let Some(c) = &out.category_id
            && may_override
            && !f.has_splits
            && f.category_id.as_ref() != Some(c)
        {
            p.category_id = Some(Some(c.clone()));
        }
        if let Some(m) = &out.member_id
            && (f.member_id.is_none() || !only_uncategorized)
            && f.member_id.as_ref() != Some(m)
        {
            p.member_id = Some(m.clone());
        }
        if let Some(name) = &out.rename_to
            && *name != f.description
        {
            p.description = Some(name.clone());
        }
        if let Some(tags) = &out.tags {
            let have: Vec<String> = existing_tags
                .query_map([&f.id], |r| r.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            p.add_tags = tags
                .iter()
                .filter(|t| !have.iter().any(|h| h.eq_ignore_ascii_case(t)))
                .cloned()
                .collect();
        }
        let changed = p.category_id.is_some()
            || p.member_id.is_some()
            || p.description.is_some()
            || p.kind.is_some()
            || !p.add_tags.is_empty();
        if changed {
            changes.push(p);
        }
    }
    Ok(Plan { matched, changes })
}

fn write_plan(tx: &Transaction<'_>, changes: &[Planned]) -> Result<HashMap<Id, i64>> {
    let now = now_utc();
    let mut hits: HashMap<Id, i64> = HashMap::new();
    let mut tags_of = tx.prepare_cached(
        "SELECT g.name FROM txn_tag tt JOIN tag g ON g.id = tt.tag_id WHERE tt.txn_id = ?1 ORDER BY g.name",
    )?;
    for p in changes {
        if let Some(c) = &p.category_id {
            tx.execute(
                "UPDATE txn SET category_id = ?2, updated_at = ?3 WHERE id = ?1",
                params![p.id, c, now],
            )?;
        }
        if let Some(m) = &p.member_id {
            tx.execute(
                "UPDATE txn SET member_id = ?2, updated_at = ?3 WHERE id = ?1",
                params![p.id, m, now],
            )?;
        }
        if let Some(d) = &p.description {
            tx.execute(
                "UPDATE txn SET description = ?2, updated_at = ?3 WHERE id = ?1",
                params![p.id, d, now],
            )?;
        }
        if let Some(k) = p.kind {
            tx.execute(
                "UPDATE txn SET kind = ?2, updated_at = ?3 WHERE id = ?1",
                params![p.id, k.as_str(), now],
            )?;
        }
        if !p.add_tags.is_empty() {
            let mut names: Vec<String> = tags_of
                .query_map([&p.id], |r| r.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            names.extend(p.add_tags.iter().cloned());
            crate::transactions::replace_tags(tx, &p.id, &names)?;
        }
        for r in &p.rules {
            *hits.entry(r.clone()).or_default() += 1;
        }
    }
    record_hits(tx, &hits)?;
    Ok(hits)
}

/// Visão "como ficaria" para a simulação.
fn simulate_views(conn: &Connection, changes: &[Planned]) -> Result<Vec<TxnView>> {
    let ids: Vec<Id> = changes.iter().take(50).map(|p| p.id.clone()).collect();
    let mut views = views::load(conn, &ids)?;
    let paths = views::category_paths(conn)?;
    let mut member_name = conn.prepare_cached("SELECT name FROM member WHERE id = ?1")?;
    for v in &mut views {
        let Some(p) = changes.iter().find(|p| p.id == v.id) else {
            continue;
        };
        if let Some(c) = &p.category_id {
            v.category_id = c.clone();
            v.category_path = c.as_ref().and_then(|c| paths.get(c).cloned());
        }
        if let Some(m) = &p.member_id {
            v.member_id = Some(m.clone());
            v.member_name = member_name.query_row([m], |r| r.get(0)).optional()?;
        }
        if let Some(d) = &p.description {
            v.description = d.clone();
        }
        if let Some(k) = p.kind {
            v.kind = k;
        }
        for t in &p.add_tags {
            v.tags.push(crate::transactions::Tag {
                id: String::new(),
                name: t.clone(),
                color: None,
            });
        }
    }
    Ok(views)
}

/// `rules.apply`: simula (`dry_run`) ou aplica num único conjunto de alterações.
pub fn apply(ctx: &mut crate::api::Ctx<'_>, args: &RuleApplyArgs) -> Result<RuleApplyResult> {
    let plan = plan(ctx.conn(), args)?;
    if args.dry_run.unwrap_or(false) || plan.changes.is_empty() {
        return Ok(RuleApplyResult {
            matched: plan.matched,
            updated: plan.changes.len() as i64,
            sample: simulate_views(ctx.conn(), &plan.changes)?,
            change_set_id: None,
        });
    }
    let changes = plan.changes;
    let rec = ctx.write(&t("nucleo-lancamentos-historico-regras-aplicadas"), |tx| {
        write_plan(tx, &changes)
    })?;
    let ids: Vec<Id> = changes.iter().take(50).map(|p| p.id.clone()).collect();
    Ok(RuleApplyResult {
        matched: plan.matched,
        updated: changes.len() as i64,
        sample: views::load(ctx.conn(), &ids)?,
        change_set_id: rec.change_set_id,
    })
}

/// Palavras que não identificam o estabelecimento (meios de pagamento, preposições,
/// sufixos societários).
const NOISE: &[&str] = &[
    "a",
    "o",
    "e",
    "as",
    "os",
    "de",
    "da",
    "do",
    "das",
    "dos",
    "em",
    "no",
    "na",
    "nos",
    "nas",
    "para",
    "pra",
    "pelo",
    "pela",
    "com",
    "por",
    "ao",
    "compra",
    "compras",
    "pag",
    "pagto",
    "pgto",
    "pagamento",
    "pagamentos",
    "debito",
    "credito",
    "cartao",
    "card",
    "pix",
    "ted",
    "doc",
    "tef",
    "transf",
    "transferencia",
    "enviado",
    "enviada",
    "recebido",
    "recebida",
    "boleto",
    "int",
    "internet",
    "online",
    "app",
    "ltda",
    "me",
    "sa",
    "eireli",
    "epp",
    "mei",
    "cia",
    "br",
    "bra",
    "brasil",
    "com",
    "www",
    "http",
    "https",
    "net",
    "org",
    "parcela",
    "parc",
    "aut",
    "automatico",
    "deb",
    "cred",
    "visa",
    "master",
    "mastercard",
    "elo",
    "nacional",
];

/// Palavra que ajuda a identificar o estabelecimento (não é meio de pagamento,
/// preposição, sufixo societário nem só dígitos).
pub(crate) fn significant(w: &str) -> bool {
    w.len() >= 3 && !w.chars().all(|c| c.is_ascii_digit()) && !NOISE.contains(&w)
}

/// Trecho mais significativo de uma descrição: a primeira sequência de até 3 palavras
/// relevantes (ver [`NOISE`]).
pub fn significant_run(description: &str) -> Vec<String> {
    let norm = crate::transactions::normalize_description(description);
    let words: Vec<&str> = norm.split(' ').filter(|w| !w.is_empty()).collect();
    let Some(start) = words.iter().position(|w| significant(w)) else {
        return words
            .iter()
            .filter(|w| w.len() >= 2)
            .take(2)
            .map(|w| w.to_string())
            .collect();
    };
    words[start..]
        .iter()
        .take_while(|w| significant(w))
        .take(3)
        .map(|w| w.to_string())
        .collect()
}

/// `rules.suggest_from_txn`: "descrição contém <trecho>" + a categoria do lançamento.
/// O trecho é o menor prefixo do trecho significativo que não pegaria lançamentos já
/// categorizados de outra forma.
pub fn suggest_from_txn(conn: &Connection, txn_id: &str) -> Result<RuleInput> {
    let f = facts_for(conn, Some(&[txn_id.to_string()]))?
        .into_iter()
        .next()
        .ok_or_else(|| Error::not_found(crate::transactions::util::txn_ref(txn_id)))?;
    let run = significant_run(&f.description);
    if run.is_empty() {
        return Err(Error::invalid(t(
            "nucleo-lancamentos-regra-erro-sem-palavras",
        )));
    }
    let mut stmt = conn.prepare(
        "SELECT description, category_id FROM txn WHERE category_id IS NOT NULL AND id <> ?1",
    )?;
    let others: Vec<(String, Id)> = stmt
        .query_map([txn_id], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let others: Vec<(String, Id)> = others.into_iter().map(|(d, c)| (fold(&d), c)).collect();
    let mut chosen = run.join(" ");
    for n in 1..=run.len() {
        let candidate = run[..n].join(" ");
        if candidate.len() < 4 && n < run.len() {
            continue;
        }
        let conflicts = others
            .iter()
            .any(|(d, c)| d.contains(&candidate) && Some(c) != f.category_id.as_ref());
        if !conflicts {
            chosen = candidate;
            break;
        }
    }
    let label = chosen.to_uppercase();
    Ok(RuleInput {
        name: t_args(
            "nucleo-lancamentos-regra-nome-sugerido",
            &args(&[("trecho", label.into())]),
        ),
        priority: Some(100),
        active: Some(true),
        match_all: Some(true),
        conditions: vec![RuleCondition {
            field: RuleField::Description,
            op: RuleOp::Contains,
            value: serde_json::Value::String(chosen),
            value2: None,
        }],
        actions: RuleActions {
            category_id: if f.kind.is_transfer_like() {
                None
            } else {
                f.category_id.clone()
            },
            ..Default::default()
        },
    })
}

#[cfg(test)]
mod tests;
