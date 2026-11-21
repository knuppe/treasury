//! Transferências entre contas próprias que chegaram como dois lançamentos soltos (ex.: o
//! salário cai numa conta e parte vai para outras; cada extrato traz a sua perna). Sem
//! ligar as pernas, a saída conta como despesa e a entrada como receita de novo.
//!
//! - [`candidates`]: pares saída/entrada de mesmo valor e moeda, em contas diferentes, com
//!   até [`MAX_DAYS`] dias de distância, ainda não ligados; cada lançamento entra num par só
//!   (o mais provável). Pares que o usuário disse que não são transferência ficam de fora.
//! - [`link`]: liga o par como transferência (ou pagamento de fatura, se o destino é cartão;
//!   ou aporte/resgate, se uma das contas é de corretora), sem categoria, com o mesmo
//!   `transfer_id`. Desfazer pelo histórico volta ao que era.
//! - [`unlink`]: desfaz a ligação (as duas pernas voltam a ficar soltas); o par não volta
//!   como sugestão.
//! - Contas "sempre ligar": depois que o usuário confirma um par entre duas contas, ele pode
//!   pedir que as próximas entre elas sejam ligadas sozinhas na importação ([`auto_link`]).

use std::collections::HashSet;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::TxnKind;
use crate::accounts::AccountKind;
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::now_utc;

/// Distância máxima, em dias, entre as duas pernas.
pub const MAX_DAYS: i64 = 4;
/// Pares recusados (`saída|entrada`).
pub const DISMISSED_SETTING: &str = "transfers.dismissed";
/// Pares de contas cujas transferências são ligadas sozinhas (`contaA|contaB`, em ordem).
pub const AUTO_SETTING: &str = "transfers.auto_accounts";
/// Nota mínima para ligar sozinho na importação (entre contas marcadas "sempre ligar").
const AUTO_MIN_SCORE: i64 = 45;

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[ts(export)]
pub struct TransferCandidatesArgs {
    /// Só lançamentos dos últimos N dias (padrão 180).
    #[serde(default)]
    #[ts(optional)]
    pub days: Option<u32>,
    /// Só os que envolvem estes lançamentos (ex.: os recém-importados).
    #[serde(default)]
    #[ts(optional)]
    pub txn_ids: Option<Vec<Id>>,
    #[serde(default)]
    #[ts(optional)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TransferLeg {
    pub txn_id: Id,
    pub account_id: Id,
    pub account_name: String,
    pub date: String,
    pub description: String,
    /// Categoria atual (some ao ligar).
    pub category_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TransferCandidate {
    /// Perna que sai (valor negativo).
    pub from: TransferLeg,
    /// Perna que entra (valor positivo).
    pub to: TransferLeg,
    /// Valor transferido (positivo, na moeda das contas).
    #[ts(type = "number")]
    pub amount: i64,
    pub currency: String,
    /// Como o par seria ligado: transferência, pagamento de fatura ou aporte.
    pub kind: TxnKind,
    #[ts(type = "number")]
    pub days_apart: i64,
    /// 0 a 100: mesma data e textos de transferência (PIX, TED…) pesam a favor.
    #[ts(type = "number")]
    pub score: i64,
    /// Por que parece transferência, em uma frase.
    pub reason: String,
    /// As próximas entre estas duas contas já são ligadas sozinhas.
    pub auto: bool,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct TransferPair {
    pub from_id: Id,
    pub to_id: Id,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct LinkTransfersArgs {
    pub pairs: Vec<TransferPair>,
    /// Ligar sozinho, na importação, as próximas entre as mesmas contas.
    #[serde(default)]
    #[ts(optional)]
    pub remember: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LinkTransfersResult {
    #[ts(type = "number")]
    pub linked: u32,
    pub change_set_id: Option<Id>,
}

struct Leg {
    id: Id,
    account_id: Id,
    account_name: String,
    account_kind: AccountKind,
    date: String,
    amount: i64,
    currency: String,
    description: String,
    category_path: Option<String>,
}

const KEYWORDS: &[&str] = &[
    "pix",
    "ted",
    "doc",
    "tef",
    "transf",
    "transferencia",
    "envio",
    "enviado",
    "recebido",
    "resgate",
    "aplicacao",
    "aplic",
    "deposito",
    "saque",
];

/// Estorno, reembolso, IOF devolvido, cashback: crédito no cartão que não é pagamento da
/// fatura ("IOF de volta de X" não paga a fatura com o "IOF de X" de outra conta).
const REFUND_WORDS: &[&str] = &[
    "de volta",
    "estorno",
    "estornado",
    "reembolso",
    "devolucao",
    "cashback",
    "credito de iof",
    "iof",
];

fn is_refund(text: &str) -> bool {
    let t = format!(" {} ", crate::import::text::fold(text));
    REFUND_WORDS.iter().any(|w| t.contains(&format!(" {w} ")))
}

fn has_keyword(text: &str) -> bool {
    let t = crate::import::text::fold(text);
    t.split(|c: char| !c.is_alphanumeric()).any(|w| {
        KEYWORDS
            .iter()
            .any(|k| w == *k || (k.len() > 4 && w.starts_with(k)))
    })
}

fn pair_key(a: &str, b: &str) -> String {
    format!("{a}|{b}")
}

fn accounts_key(a: &str, b: &str) -> String {
    if a <= b {
        format!("{a}|{b}")
    } else {
        format!("{b}|{a}")
    }
}

/// Tipo do par pelas contas: destino cartão é pagamento de fatura; corretora ou conta de
/// aposentadoria de um lado, aporte ou resgate; o resto, transferência.
fn kind_for(from: AccountKind, to: AccountKind) -> TxnKind {
    if to == AccountKind::CreditCard {
        TxnKind::CardPayment
    } else if [from, to]
        .iter()
        .any(|k| matches!(k, AccountKind::Brokerage | AccountKind::Retirement))
    {
        TxnKind::Investment
    } else {
        TxnKind::Transfer
    }
}

fn legs(conn: &Connection, since: &str) -> Result<Vec<Leg>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.account_id, a.name, a.kind, t.date, t.amount, t.currency, t.description,
                (SELECT CASE WHEN p.name IS NULL THEN c.name ELSE p.name || ' › ' || c.name END
                   FROM category c LEFT JOIN category p ON p.id = c.parent_id WHERE c.id = t.category_id)
           FROM txn t JOIN account a ON a.id = t.account_id
          WHERE t.date >= ?1 AND t.transfer_id IS NULL AND t.kind IN ('income', 'expense', 'transfer')
            -- Cópia de um lançamento de outra conta (o mesmo identificador do banco, como o
            -- arquivo do cartão importado na conta à vista) não é perna de transferência.
            AND NOT (t.external_id IS NOT NULL AND t.external_id <> '' AND EXISTS (
                SELECT 1 FROM txn o WHERE o.external_id = t.external_id
                   AND o.account_id <> t.account_id))
            AND t.status <> 'planned' AND t.amount <> 0 AND t.installment_plan_id IS NULL
            AND a.archived = 0
            AND NOT EXISTS (SELECT 1 FROM txn_split s WHERE s.txn_id = t.id)
          ORDER BY t.date, t.id",
    )?;
    let rows = stmt.query_map([since], |r| {
        Ok(Leg {
            id: r.get(0)?,
            account_id: r.get(1)?,
            account_name: r.get(2)?,
            account_kind: AccountKind::parse(&r.get::<_, String>(3)?).unwrap_or(AccountKind::Other),
            date: r.get(4)?,
            amount: r.get(5)?,
            currency: r.get(6)?,
            description: r.get(7)?,
            category_path: r.get(8)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

fn days_between(a: &str, b: &str) -> i64 {
    match (
        NaiveDate::parse_from_str(a, "%Y-%m-%d"),
        NaiveDate::parse_from_str(b, "%Y-%m-%d"),
    ) {
        (Ok(x), Ok(y)) => (x - y).num_days().abs(),
        _ => i64::MAX,
    }
}

fn to_leg(l: &Leg) -> TransferLeg {
    TransferLeg {
        txn_id: l.id.clone(),
        account_id: l.account_id.clone(),
        account_name: l.account_name.clone(),
        date: l.date.clone(),
        description: l.description.clone(),
        category_path: l.category_path.clone(),
    }
}

/// Pares prováveis, do mais provável para o menos; cada lançamento aparece uma vez.
pub fn candidates(
    conn: &Connection,
    a: &TransferCandidatesArgs,
    today: NaiveDate,
) -> Result<Vec<TransferCandidate>> {
    let since = (today - chrono::Duration::days(a.days.unwrap_or(180).clamp(1, 3650) as i64))
        .format("%Y-%m-%d")
        .to_string();
    let only: Option<HashSet<Id>> = a.txn_ids.as_ref().map(|v| v.iter().cloned().collect());
    // O filtro vale para o par (as contrapartes podem ser antigas), não para a perna.
    let all = legs(conn, &since)?;
    let dismissed: HashSet<String> =
        crate::settings::get_or::<Vec<String>>(conn, DISMISSED_SETTING, vec![])?
            .into_iter()
            .collect();
    let auto: HashSet<String> = crate::settings::get_or::<Vec<String>>(conn, AUTO_SETTING, vec![])?
        .into_iter()
        .collect();
    let (outs, ins): (Vec<&Leg>, Vec<&Leg>) = all.iter().partition(|l| l.amount < 0);
    let mut found: Vec<TransferCandidate> = vec![];
    for o in &outs {
        for i in &ins {
            if i.account_id == o.account_id || i.currency != o.currency || i.amount != -o.amount {
                continue;
            }
            if only
                .as_ref()
                .is_some_and(|s| !s.contains(&o.id) && !s.contains(&i.id))
            {
                continue;
            }
            let gap = days_between(&o.date, &i.date);
            if gap > MAX_DAYS || dismissed.contains(&pair_key(&o.id, &i.id)) {
                continue;
            }
            let kind = kind_for(o.account_kind, i.account_kind);
            // Pagamento de fatura: o crédito no cartão não pode ser estorno ou IOF devolvido, e
            // sem texto de pagamento ("Pagamento de fatura", "Pagamento recebido") o par só é
            // sugerido, nunca ligado sozinho.
            let bill_text = crate::import::is_bill_payment(&o.description, false)
                || crate::import::is_bill_payment(&i.description, true);
            if kind == TxnKind::CardPayment
                && (is_refund(&i.description) || is_refund(&o.description))
            {
                continue;
            }
            let words = has_keyword(&o.description) || has_keyword(&i.description);
            // O nome da outra conta no texto (ex.: "PIX para Nubank") é um sinal forte.
            let named = crate::import::text::fold(&o.description)
                .contains(&crate::import::text::fold(&i.account_name))
                || crate::import::text::fold(&i.description)
                    .contains(&crate::import::text::fold(&o.account_name));
            let mut score =
                (70 - gap * 12 + if words { 20 } else { 0 } + if named { 15 } else { 0 })
                    .clamp(5, 100);
            if kind == TxnKind::CardPayment && !bill_text {
                score = score.min(AUTO_MIN_SCORE - 5);
            }
            let mut why = vec![if gap == 0 {
                t("nucleo-lancamentos-motivo-mesmo-valor-mesmo-dia")
            } else {
                t_args(
                    "nucleo-lancamentos-motivo-mesmo-valor-dias",
                    &args(&[("dias", gap.into())]),
                )
            }];
            if words {
                why.push(t("nucleo-lancamentos-motivo-texto-transferencia"));
            }
            if named {
                why.push(t("nucleo-lancamentos-motivo-nome-outra-conta"));
            }
            found.push(TransferCandidate {
                from: to_leg(o),
                to: to_leg(i),
                amount: -o.amount,
                currency: o.currency.clone(),
                kind,
                days_apart: gap,
                score,
                reason: why.join("; "),
                auto: auto.contains(&accounts_key(&o.account_id, &i.account_id)),
            });
        }
    }
    found.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(a.days_apart.cmp(&b.days_apart))
            .then(b.from.date.cmp(&a.from.date))
    });
    let mut used: HashSet<Id> = HashSet::new();
    let mut out = vec![];
    for c in found {
        if used.contains(&c.from.txn_id) || used.contains(&c.to.txn_id) {
            continue;
        }
        used.insert(c.from.txn_id.clone());
        used.insert(c.to.txn_id.clone());
        out.push(c);
    }
    out.truncate(a.limit.unwrap_or(200).clamp(1, 1000) as usize);
    Ok(out)
}

/// (conta, valor, moeda, transferência, tipo, fatura) de uma perna.
type LegRow = (String, i64, String, Option<String>, String, Option<String>);

/// Liga duas pernas soltas como transferência. Erro claro se não formam um par.
pub fn link_pair(tx: &Transaction<'_>, from_id: &str, to_id: &str, today: NaiveDate) -> Result<()> {
    let read = |id: &str| -> Result<LegRow> {
        tx.query_row(
            "SELECT account_id, amount, currency, transfer_id, kind, statement_id FROM txn WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
        )
        .optional()?
        .ok_or_else(|| Error::not_found(super::util::txn_ref(id)))
    };
    let (fa, famt, fcur, ftid, fkind, _) = read(from_id)?;
    let (ta, tamt, tcur, ttid, tkind, tstmt) = read(to_id)?;
    if ftid.is_some() || ttid.is_some() {
        return Err(Error::conflict(t(
            "nucleo-lancamentos-erro-par-ja-transferencia",
        )));
    }
    let loose = |k: &str| matches!(k, "income" | "expense" | "transfer");
    if !loose(&fkind) || !loose(&tkind) {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-par-tipo")));
    }
    if fa == ta {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-par-mesma-conta")));
    }
    if famt >= 0 || tamt <= 0 {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-par-sinal")));
    }
    if fcur == tcur && -famt != tamt {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-par-valor")));
    }
    for id in [from_id, to_id] {
        let splits: bool = tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM txn_split WHERE txn_id = ?1)",
            [id],
            |r| r.get(0),
        )?;
        if splits {
            return Err(Error::invalid(t("nucleo-lancamentos-erro-par-dividido")));
        }
    }
    let (_, fk) = crate::accounts::currency_and_kind(tx, &fa)?;
    let (_, tk) = crate::accounts::currency_and_kind(tx, &ta)?;
    let kind = kind_for(fk, tk);
    let tid = new_id();
    let now = now_utc();
    for id in [from_id, to_id] {
        tx.execute(
            // A categoria só fica em transferência (informativa); fatura e aporte não usam.
            "UPDATE txn SET kind = ?2, transfer_id = ?3, updated_at = ?4,
                    category_id = CASE WHEN ?2 = 'transfer' THEN category_id END
              WHERE id = ?1",
            params![id, kind.as_str(), tid, now],
        )?;
    }
    if kind == TxnKind::CardPayment {
        let date: String =
            tx.query_row("SELECT date FROM txn WHERE id = ?1", [to_id], |r| r.get(0))?;
        let stmt = match tstmt {
            Some(s) => Some(s),
            None => crate::cards::statement_to_pay(tx, &ta, &date)?,
        };
        if let Some(s) = &stmt {
            tx.execute(
                "UPDATE txn SET statement_id = ?2 WHERE id = ?1",
                params![to_id, s],
            )?;
            crate::cards::sync_statuses(tx, std::slice::from_ref(s), today)?;
        }
    }
    Ok(())
}

/// Valor da outra perna: o oposto, na mesma moeda; em outra moeda, o informado (com o sinal
/// oposto) ou `None` se faltar.
fn other_leg_amount(
    amount: i64,
    currency: &str,
    other: &str,
    informed: Option<i64>,
) -> Option<i64> {
    if currency == other {
        Some(-amount)
    } else {
        informed
            .filter(|v| *v != 0)
            .map(|v| v.abs() * -amount.signum())
    }
}

/// Lançamento solto da outra conta que forma o par: valor `amount`, até [`MAX_DAYS`] dias
/// da data, sem transferência nem divisão; o de data mais próxima.
fn counterpart(
    conn: &Connection,
    account_id: &str,
    amount: i64,
    date: &str,
    except: &str,
) -> Result<Option<Id>> {
    Ok(conn
        .query_row(
            "SELECT t.id FROM txn t
              WHERE t.account_id = ?1 AND t.amount = ?2 AND t.transfer_id IS NULL AND t.id <> ?5
                AND t.kind IN ('income', 'expense', 'transfer') AND t.installment_plan_id IS NULL
                AND abs(julianday(t.date) - julianday(?3)) <= ?4
                AND NOT EXISTS (SELECT 1 FROM txn_split s WHERE s.txn_id = t.id)
              ORDER BY abs(julianday(t.date) - julianday(?3)), t.id LIMIT 1",
            params![account_id, amount, date, MAX_DAYS, except],
            |r| r.get(0),
        )
        .optional()?)
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct TransferMatchArgs {
    pub txn_id: Id,
    /// A outra conta da família.
    pub account_id: Id,
    /// Valor na outra conta, quando ela usa outra moeda.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub other_amount: Option<i64>,
}

/// O que acontece ao ligar o lançamento à outra conta ([`link_to_account`]), sem gravar.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TransferMatch {
    /// A perna que já existe na outra conta (o par é ligado a ela); `null` = ela será criada.
    pub existing: Option<TransferLeg>,
    /// Valor da outra perna, com sinal (`null` se a moeda é outra e falta o valor).
    #[ts(type = "number | null")]
    pub amount: Option<i64>,
    pub currency: String,
    /// O que o par vira: transferência, pagamento de fatura ou aporte/resgate.
    pub kind: TxnKind,
    /// Vencimento da fatura que o pagamento quita (a outra conta é cartão).
    pub statement_due_date: Option<String>,
}

pub fn match_account(conn: &Connection, a: &TransferMatchArgs) -> Result<TransferMatch> {
    let row = super::load_row(conn, &a.txn_id)?;
    if row.transfer_id.is_some() {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-ja-ligado")));
    }
    if row.account_id == a.account_id {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-outra-conta")));
    }
    let (currency, other_kind) = crate::accounts::currency_and_kind(conn, &a.account_id)?;
    let (_, own_kind) = crate::accounts::currency_and_kind(conn, &row.account_id)?;
    let amount = other_leg_amount(row.amount, &row.currency, &currency, a.other_amount);
    let existing = match amount {
        Some(v) => counterpart(conn, &a.account_id, v, &row.date, &row.id)?,
        None => None,
    };
    let existing = match existing {
        Some(id) => Some(conn.query_row(
            "SELECT t.id, t.account_id, ac.name, t.date, t.description, t.category_id, t.statement_id
               FROM txn t JOIN account ac ON ac.id = t.account_id WHERE t.id = ?1",
            [&id],
            |r| {
                Ok((
                    TransferLeg {
                        txn_id: r.get(0)?,
                        account_id: r.get(1)?,
                        account_name: r.get(2)?,
                        date: r.get(3)?,
                        description: r.get(4)?,
                        category_path: None,
                    },
                    r.get::<_, Option<Id>>(5)?,
                    r.get::<_, Option<Id>>(6)?,
                ))
            },
        )?),
        None => None,
    };
    let kind = if row.amount < 0 {
        kind_for(own_kind, other_kind)
    } else {
        kind_for(other_kind, own_kind)
    };
    // Pagamento de fatura: a fatura da perna do cartão (a da perna que existe ou a que a
    // data dela paga).
    let statement_due_date = if kind == TxnKind::CardPayment {
        let (card, date, stmt) = match &existing {
            Some((leg, _, stmt)) if row.amount < 0 => {
                (a.account_id.clone(), leg.date.clone(), stmt.clone())
            }
            _ if row.amount < 0 => (a.account_id.clone(), row.date.clone(), None),
            _ => (
                row.account_id.clone(),
                row.date.clone(),
                row.statement_id.clone(),
            ),
        };
        let stmt = match stmt {
            Some(s) => Some(s),
            None => crate::cards::statement_to_pay(conn, &card, &date)?,
        };
        match stmt {
            Some(s) => conn
                .query_row(
                    "SELECT due_date FROM card_statement WHERE id = ?1",
                    [s],
                    |r| r.get(0),
                )
                .optional()?,
            None => None,
        }
    } else {
        None
    };
    let paths = crate::categories::path_map(conn)?;
    Ok(TransferMatch {
        existing: existing.map(|(mut leg, category, _)| {
            leg.category_path = category.and_then(|c| paths.get(&c).cloned());
            leg
        }),
        amount,
        currency,
        kind,
        statement_due_date,
    })
}

/// Liga um lançamento solto à outra conta da família: usa a perna que já existe lá (valor
/// oposto, sem transferência, até alguns dias de diferença, a mais próxima) ou cria essa
/// perna (mesma data, descrição e status). Devolve o id da outra perna.
pub fn link_to_account(
    tx: &Transaction<'_>,
    txn_id: &str,
    account_id: &str,
    other_amount: Option<i64>,
    today: NaiveDate,
) -> Result<Id> {
    let row = super::load_row(tx, txn_id)?;
    if row.transfer_id.is_some() {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-ja-ligado")));
    }
    super::operations::ensure_free(tx, txn_id)?;
    if row.account_id == account_id {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-outra-conta")));
    }
    let (currency, _) = crate::accounts::currency_and_kind(tx, account_id)?;
    let Some(amount) = other_leg_amount(row.amount, &row.currency, &currency, other_amount) else {
        return Err(Error::invalid(t_args(
            "nucleo-lancamentos-erro-outra-conta-moeda",
            &args(&[("moeda", currency.into())]),
        )));
    };
    let found = counterpart(tx, account_id, amount, &row.date, txn_id)?;
    let other = match found {
        Some(id) => id,
        None => super::create(
            tx,
            &super::TxnInput {
                account_id: account_id.to_string(),
                date: row.date.clone(),
                amount,
                kind: Some(TxnKind::Transfer),
                status: Some(row.status),
                description: row.description.clone(),
                payee: None,
                notes: None,
                category_id: None,
                member_id: row.member_id.clone(),
                splits: None,
                tags: None,
                project_id: None,
                card_plastic_id: None,
                orig_amount: None,
                orig_currency: None,
            },
            today,
        )?,
    };
    if row.amount < 0 {
        link_pair(tx, txn_id, &other, today)?;
    } else {
        link_pair(tx, &other, txn_id, today)?;
    }
    Ok(other)
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct UnlinkTransferArgs {
    /// Uma das pernas da transferência.
    pub txn_id: Id,
    /// Não sugerir de novo este par (padrão: sim).
    #[serde(default)]
    #[ts(optional)]
    pub dismiss: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UnlinkTransferResult {
    /// As pernas que ficaram soltas (saída primeiro).
    pub txn_ids: Vec<Id>,
    pub change_set_id: Option<Id>,
}

/// Desfaz a ligação de uma transferência (ou pagamento de fatura, ou aporte/resgate): as
/// duas pernas ficam soltas, como transferência sem a outra conta (fora de receitas e
/// despesas enquanto sem categoria), prontas para ligar a outra perna ou virar receita ou
/// despesa. A categoria informativa fica. Devolve as pernas, a saída primeiro.
pub fn unlink(tx: &Transaction<'_>, txn_id: &str, today: NaiveDate) -> Result<Vec<Id>> {
    let tid: Option<String> = tx
        .query_row("SELECT transfer_id FROM txn WHERE id = ?1", [txn_id], |r| {
            r.get(0)
        })
        .optional()?
        .ok_or_else(|| Error::not_found(super::util::txn_ref(txn_id)))?;
    let Some(tid) = tid else {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-nao-ligado")));
    };
    // (id, conta, data, valor, tipo, fatura)
    let legs: Vec<(Id, Id, String, i64, String, Option<Id>)> = {
        let mut stmt = tx.prepare(
            "SELECT id, account_id, date, amount, kind, statement_id FROM txn
              WHERE transfer_id = ?1 ORDER BY amount, id",
        )?;
        stmt.query_map([&tid], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?
    };
    if let Some((_, _, _, _, k, _)) = legs
        .iter()
        .find(|l| !matches!(l.4.as_str(), "transfer" | "card_payment" | "investment"))
    {
        return Err(Error::invalid(t_args(
            "nucleo-lancamentos-erro-ligacao-historico",
            &args(&[("tipo", k.clone().into())]),
        )));
    }
    let now = now_utc();
    let mut touched: Vec<Id> = vec![];
    for (id, account, date, _, _, stmt) in &legs {
        // A perna do cartão volta para a fatura da data (deixa de quitar uma fatura).
        let new_stmt = crate::cards::statement_for(tx, account, date)?;
        touched.extend(stmt.iter().cloned());
        touched.extend(new_stmt.iter().cloned());
        tx.execute(
            "UPDATE txn SET kind = 'transfer', transfer_id = NULL, statement_id = ?2, updated_at = ?3
              WHERE id = ?1",
            params![id, new_stmt, now],
        )?;
    }
    touched.sort();
    touched.dedup();
    crate::cards::sync_statuses(tx, &touched, today)?;
    Ok(legs.into_iter().map(|l| l.0).collect())
}

pub fn remember_accounts(conn: &Connection, pairs: &[(String, String)]) -> Result<()> {
    let mut auto: Vec<String> = crate::settings::get_or(conn, AUTO_SETTING, vec![])?;
    for (a, b) in pairs {
        let k = accounts_key(a, b);
        if !auto.contains(&k) {
            auto.push(k);
        }
    }
    crate::settings::set(conn, AUTO_SETTING, &auto)
}

pub fn forget_accounts(conn: &Connection, a: &str, b: &str) -> Result<()> {
    let mut auto: Vec<String> = crate::settings::get_or(conn, AUTO_SETTING, vec![])?;
    auto.retain(|k| *k != accounts_key(a, b));
    crate::settings::set(conn, AUTO_SETTING, &auto)
}

pub fn dismiss(conn: &Connection, p: &TransferPair) -> Result<()> {
    let mut d: Vec<String> = crate::settings::get_or(conn, DISMISSED_SETTING, vec![])?;
    let k = pair_key(&p.from_id, &p.to_id);
    if !d.contains(&k) {
        d.push(k);
        // Guarda só os mais recentes.
        if d.len() > 2000 {
            d.drain(..d.len() - 2000);
        }
    }
    crate::settings::set(conn, DISMISSED_SETTING, &d)
}

/// Depois de importar: liga sozinhos os pares entre contas marcadas como "sempre ligar"
/// (o mais provável de cada lançamento novo, com nota alta). Devolve quantos ligou.
pub fn auto_link(tx: &Transaction<'_>, new_ids: &[Id], today: NaiveDate) -> Result<u32> {
    let auto: Vec<String> = crate::settings::get_or(tx, AUTO_SETTING, vec![])?;
    if auto.is_empty() || new_ids.is_empty() {
        return Ok(0);
    }
    let found = candidates(
        tx,
        &TransferCandidatesArgs {
            days: Some(400),
            txn_ids: Some(new_ids.to_vec()),
            limit: Some(1000),
        },
        today,
    )?;
    let mut n = 0;
    for c in found
        .into_iter()
        .filter(|c| c.auto && c.score >= AUTO_MIN_SCORE)
    {
        link_pair(tx, &c.from.txn_id, &c.to.txn_id, today)?;
        n += 1;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::api::Api;
    use crate::paths::TreasuryPaths;

    fn api() -> (tempfile::TempDir, Api, String, String) {
        let dir = tempfile::tempdir().unwrap();
        let api =
            Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
                .unwrap();
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
        )
        .unwrap();
        let acc = |name: &str| {
            api.call(
                "accounts.create",
                json!({"name": name, "kind": "checking", "opening_date": "2026-01-01"}),
            )
            .unwrap()["id"]
                .as_str()
                .unwrap()
                .to_string()
        };
        let (a, b) = (acc("Itaú"), acc("Nubank"));
        (dir, api, a, b)
    }

    fn txn(api: &Api, acc: &str, date: &str, amount: i64, desc: &str) -> String {
        api.call(
            "transactions.create",
            json!({"account_id": acc, "date": date, "amount": amount, "description": desc}),
        )
        .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn expense_of(api: &Api) -> i64 {
        api.call(
            "reports.month_flow",
            json!({"from_month": "2026-10", "to_month": "2026-10", "basis": "accrual"}),
        )
        .unwrap()[0]["expense"]
            .as_i64()
            .unwrap()
    }

    #[test]
    fn expense_becomes_transfer_counting_by_category_or_linked_to_own_account() {
        let (_d, api, a, b) = api();
        let cat = api
            .call("categories.list", json!({}))
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["kind"] == "expense" && !c["parent_id"].is_null())
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string();
        let pix = txn(&api, &a, "2026-10-05", -5000, "COMPRA LOJA");
        assert_eq!(expense_of(&api), 5000);
        // Transfer without a category: outside the flow.
        api.call(
            "transactions.update",
            json!({"id": pix, "kind": "transfer", "payee": "João"}),
        )
        .unwrap();
        assert_eq!(expense_of(&api), 0);
        // With a category (PIX to third parties): counts as an expense of the category.
        api.call(
            "transactions.update",
            json!({"id": pix, "category_id": cat}),
        )
        .unwrap();
        assert_eq!(expense_of(&api), 5000);
        // Category in batch also applies to a transfer, and it leaves "without category".
        api.call(
            "transactions.set_category",
            json!({"ids": [pix], "category_id": null}),
        )
        .unwrap();
        let pending = |api: &Api| {
            api.call("transactions.query", json!({"uncategorized": true}))
                .unwrap()["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|t| t["id"] == pix.as_str())
        };
        assert!(pending(&api));
        api.call(
            "transactions.set_category",
            json!({"ids": [pix], "category_id": cat}),
        )
        .unwrap();
        assert!(!pending(&api));

        // To another family account, without the leg there: the leg is created and linked.
        let out = txn(&api, &a, "2026-10-06", -20000, "DEBITO PIX - NUBANK");
        api.call(
            "transactions.update",
            json!({"id": out, "kind": "transfer", "transfer_account_id": b}),
        )
        .unwrap();
        let t = api.call("transactions.get", json!({"id": out})).unwrap();
        assert_eq!(t["kind"], "transfer");
        assert!(t["transfer_id"].is_string());
        assert_eq!(t["counterpart_account_id"], b.as_str());
        assert_eq!(expense_of(&api), 5000);

        // The leg already exists in the other account (imported): links to it, creates no other.
        let c_out = txn(&api, &a, "2026-10-08", -7000, "TED ENVIADA");
        let c_in = txn(&api, &b, "2026-10-09", 7000, "TED RECEBIDA");
        api.call(
            "transactions.update",
            json!({"id": c_out, "kind": "transfer", "transfer_account_id": b}),
        )
        .unwrap();
        let leg = api.call("transactions.get", json!({"id": c_in})).unwrap();
        assert_eq!(leg["kind"], "transfer");
        assert_eq!(
            leg["transfer_id"],
            api.call("transactions.get", json!({"id": c_out})).unwrap()["transfer_id"]
        );
        // Linking again is a clear error.
        assert!(
            api.call(
                "transactions.update",
                json!({"id": c_out, "transfer_account_id": b}),
            )
            .is_err()
        );
    }

    #[test]
    fn match_previews_the_pair_and_the_card_bill_before_linking() {
        let (_d, api, a, b) = api();
        let out = txn(&api, &a, "2026-10-10", -50_000, "PIX para Nubank");
        let back = txn(&api, &b, "2026-10-12", 50_000, "PIX recebido");
        txn(&api, &b, "2026-10-30", 50_000, "Longe demais");

        // The entry already exists in the other account: the pair is it.
        let m = api
            .call(
                "transactions.transfer_match",
                json!({"txn_id": out, "account_id": b}),
            )
            .unwrap();
        assert_eq!(m["existing"]["txn_id"], back.as_str());
        assert_eq!(m["kind"], "transfer");
        assert_eq!(m["amount"], 50_000);
        assert!(m["statement_due_date"].is_null());
        // In the same account, no.
        assert!(
            api.call(
                "transactions.transfer_match",
                json!({"txn_id": out, "account_id": a}),
            )
            .is_err()
        );

        // Statement payment: with no leg on the card, one is created; the statement is the due one.
        let card = api
            .call(
                "accounts.create",
                json!({"name": "Cartão", "kind": "credit_card", "opening_date": "2026-01-01",
                       "card": {"closing_day": 1, "due_day": 10}}),
            )
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string();
        txn(&api, &card, "2026-09-20", -80_000, "Mercado");
        let bill = txn(&api, &a, "2026-10-09", -80_000, "Pagamento fatura");
        let m = api
            .call(
                "transactions.transfer_match",
                json!({"txn_id": bill, "account_id": card}),
            )
            .unwrap();
        assert!(m["existing"].is_null());
        assert_eq!(m["kind"], "card_payment");
        assert_eq!(m["statement_due_date"], "2026-10-10");

        // Linking through editing does what the preview said.
        api.call(
            "transactions.update",
            json!({"id": bill, "transfer_account_id": card}),
        )
        .unwrap();
        let t = api.call("transactions.get", json!({"id": bill})).unwrap();
        assert_eq!(t["kind"], "card_payment");
        assert_eq!(t["counterpart_account_id"], card.as_str());
    }

    #[test]
    fn pix_ted_doc_rule_marks_new_and_existing_as_transfer() {
        let (_d, api, a, _b) = api();
        let t = txn(&api, &a, "2026-10-05", -5000, "PIX ENVIADO MARIA");
        let ted = txn(&api, &a, "2026-10-05", -5000, "Pagamento TED");
        let shop = txn(&api, &a, "2026-10-05", -5000, "TEDESCO LOJA");
        // Transaction created by hand: the type is the one requested (income/expense); the rule
        // applies on import and on the ready-made rule of old vaults (below).
        assert_eq!(
            api.call("transactions.get", json!({"id": t})).unwrap()["kind"],
            "expense"
        );
        let n = api
            .with_session(|s, _| {
                let tx = s.db.conn_mut().transaction()?;
                let n = crate::rules::builtin::mark_existing_transfers(&tx)?;
                tx.commit()?;
                Ok(n)
            })
            .unwrap();
        assert_eq!(n, 2);
        for (id, kind) in [(t, "transfer"), (ted, "transfer"), (shop, "expense")] {
            assert_eq!(
                api.call("transactions.get", json!({"id": id})).unwrap()["kind"],
                kind
            );
        }
        let rules = api.call("rules.list", json!(null)).unwrap();
        assert!(
            rules
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["name"] == crate::rules::builtin::TRANSFER_RULE)
        );
    }

    #[test]
    fn salary_moved_between_own_accounts_is_found_linked_and_remembered() {
        let (_d, api, itau, nu) = api();
        let today = crate::time::today_local();
        let d = |n: i64| {
            (today - chrono::Duration::days(n))
                .format("%Y-%m-%d")
                .to_string()
        };
        txn(&api, &itau, &d(10), 1_000_000, "SALARIO EMPRESA X");
        let out = txn(&api, &itau, &d(9), -300_000, "PIX ENVIADO NUBANK");
        let inn = txn(&api, &nu, &d(8), 300_000, "PIX RECEBIDO");
        // Same amount but too far away: does not enter.
        txn(&api, &itau, &d(40), -50_000, "MERCADO");
        txn(&api, &nu, &d(30), 50_000, "REEMBOLSO");
        let c = api
            .call("transactions.transfer_candidates", json!({}))
            .unwrap();
        let list = c.as_array().unwrap();
        assert_eq!(list.len(), 1, "{c}");
        assert_eq!(list[0]["from"]["txn_id"], out.as_str());
        assert_eq!(list[0]["to"]["txn_id"], inn.as_str());
        assert_eq!(list[0]["kind"], "transfer");
        assert!(list[0]["score"].as_i64().unwrap() >= 80, "{c}");
        let r = api
            .call(
                "transactions.link_transfers",
                json!({"pairs": [{"from_id": out, "to_id": inn}], "remember": true}),
            )
            .unwrap();
        assert_eq!(r["linked"], 1);
        let leg = api.call("transactions.get", json!({"id": out})).unwrap();
        assert_eq!(leg["kind"], "transfer");
        assert!(leg["category_id"].is_null());
        // Left income/expense: no longer shows up as a candidate.
        assert!(
            api.call("transactions.transfer_candidates", json!({}))
                .unwrap()
                .as_array()
                .unwrap()
                .is_empty()
        );
        // Undoing through the history returns to what it was.
        api.call("changes.undo", json!({"id": r["change_set_id"]}))
            .unwrap();
        assert_eq!(
            api.call("transactions.get", json!({"id": out})).unwrap()["kind"],
            "expense"
        );
        // Refusing the pair removes it from the list.
        api.call(
            "transactions.dismiss_transfer",
            json!({"from_id": out, "to_id": inn}),
        )
        .unwrap();
        assert!(
            api.call("transactions.transfer_candidates", json!({}))
                .unwrap()
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn pairs_must_be_opposite_equal_and_in_different_accounts() {
        let (_d, api, itau, nu) = api();
        let a = txn(&api, &itau, "2026-10-01", -1000, "x");
        let b = txn(&api, &nu, "2026-10-01", 900, "y");
        let c = txn(&api, &itau, "2026-10-01", 1000, "z");
        for (f, t) in [(&a, &b), (&a, &c), (&b, &a)] {
            assert!(
                api.call(
                    "transactions.link_transfers",
                    json!({"pairs": [{"from_id": f, "to_id": t}]}),
                )
                .is_err()
            );
        }
    }

    #[test]
    fn import_links_by_itself_between_remembered_accounts() {
        use base64::Engine as _;
        let (_d, api, itau, nu) = api();
        let out = txn(&api, &itau, "2026-10-01", -150_000, "PIX ENVIADO NUBANK");
        api.with_session(|s, _| {
            super::remember_accounts(s.db.conn(), &[(itau.clone(), nu.clone())])
        })
        .unwrap();
        let ofx = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/import/fixtures/nubank_xml.ofx"),
        )
        .unwrap();
        let r = api
            .call(
                "import.file",
                json!({"account_id": nu, "filename": "nu.ofx",
                       "content_base64": base64::engine::general_purpose::STANDARD.encode(&ofx)}),
            )
            .unwrap();
        assert_eq!(r["result"]["transfers_linked"], 1, "{r}");
        let leg = api.call("transactions.get", json!({"id": out})).unwrap();
        assert_eq!(leg["kind"], "transfer");
        assert!(leg["transfer_id"].is_string());
    }

    #[test]
    fn unlink_leaves_both_legs_loose_and_stops_suggesting_the_pair() {
        let (_d, api, itau, nu) = api();
        let today = crate::time::today_local();
        let d = |n: i64| {
            (today - chrono::Duration::days(n))
                .format("%Y-%m-%d")
                .to_string()
        };
        let out = txn(&api, &itau, &d(3), -700_000, "DEBITO PIX - ANA");
        let inn = txn(&api, &nu, &d(3), 700_000, "PIX RECEBIDO - ANA");
        api.call(
            "transactions.link_transfers",
            json!({"pairs": [{"from_id": out, "to_id": inn}]}),
        )
        .unwrap();
        let leg = api.call("transactions.get", json!({"id": inn})).unwrap();
        assert_eq!(leg["counterpart_account_id"], itau.as_str());

        // Unlinking through either leg: both are left loose, as a transfer.
        let r = api
            .call("transactions.unlink_transfer", json!({"txn_id": inn}))
            .unwrap();
        assert_eq!(r["txn_ids"], json!([out, inn]));
        assert!(r["change_set_id"].is_string());
        for id in [&out, &inn] {
            let t = api.call("transactions.get", json!({"id": id})).unwrap();
            assert!(t["transfer_id"].is_null(), "{t}");
            assert_eq!(t["kind"], "transfer");
        }
        // The unlinked pair does not come back as a suggestion; unlinking again is a clear error.
        assert!(
            api.call("transactions.transfer_candidates", json!({}))
                .unwrap()
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(
            api.call("transactions.unlink_transfer", json!({"txn_id": out}))
                .is_err()
        );
        // Through the history, the link comes back.
        api.call("changes.undo", json!({"id": r["change_set_id"]}))
            .unwrap();
        let t = api.call("transactions.get", json!({"id": out})).unwrap();
        assert_eq!(t["counterpart_account_id"], nu.as_str());
        // Linking again after unlinking (by hand) is still possible.
        api.call("transactions.unlink_transfer", json!({"txn_id": out}))
            .unwrap();
        api.call(
            "transactions.link_transfers",
            json!({"pairs": [{"from_id": out, "to_id": inn}]}),
        )
        .unwrap();
    }

    #[test]
    fn unlinking_a_card_payment_returns_the_card_leg_to_its_bill() {
        let (_d, api, a, _b) = api();
        let card = api
            .call(
                "accounts.create",
                json!({"name": "Cartão", "kind": "credit_card", "opening_date": "2026-01-01",
                       "card": {"closing_day": 1, "due_day": 10}}),
            )
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string();
        txn(&api, &card, "2026-09-20", -80_000, "Mercado");
        let bill = txn(&api, &a, "2026-10-09", -80_000, "Pagamento fatura");
        api.call(
            "transactions.update",
            json!({"id": bill, "transfer_account_id": card}),
        )
        .unwrap();
        api.call("transactions.unlink_transfer", json!({"txn_id": bill}))
            .unwrap();
        let t = api.call("transactions.get", json!({"id": bill})).unwrap();
        assert_eq!(t["kind"], "transfer");
        assert!(t["counterpart_account_id"].is_null());
    }
}
