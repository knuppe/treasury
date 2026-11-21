//! Escritas de lançamentos: criar, alterar, apagar, transferências, categoria e status em
//! lote. Todas recebem a transação de `ctx.write(...)`.

use std::collections::BTreeSet;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Transaction, params};

use super::util::{clean_opt, dedup_ids, require_member};
use super::{
    NewTxn, SplitInput, TransferInput, TxnInput, TxnKind, TxnPatch, TxnStatus, fingerprint, insert,
    replace_splits, replace_tags, validate_splits,
};
use crate::accounts::{AccountKind, currency_and_kind};
use crate::categories::CategoryKind;
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::now_utc;

/// Rótulo do tipo de lançamento (minúsculo, para o meio da frase), no idioma da instalação.
pub fn kind_label(k: TxnKind) -> String {
    t(match k {
        TxnKind::Income => "nucleo-lancamentos-tipo-receita",
        TxnKind::Expense => "nucleo-lancamentos-tipo-despesa",
        TxnKind::Transfer => "nucleo-lancamentos-tipo-transferencia",
        TxnKind::CardPayment => "nucleo-lancamentos-tipo-pagamento-fatura",
        TxnKind::Investment => "nucleo-lancamentos-tipo-investimento",
        TxnKind::Adjustment => "nucleo-lancamentos-tipo-ajuste",
    })
}

/// Rótulo do status (minúsculo), no idioma da instalação.
pub fn status_label(s: TxnStatus) -> String {
    t(match s {
        TxnStatus::Planned => "nucleo-lancamentos-status-previsto",
        TxnStatus::Cleared => "nucleo-lancamentos-status-efetivado",
        TxnStatus::Reconciled => "nucleo-lancamentos-status-conciliado",
    })
}

/// Erro de texto livre do catálogo, com um argumento.
fn invalid1(key: &str, name: &'static str, value: impl Into<String>) -> Error {
    Error::invalid(t_args(key, &args(&[(name, value.into().into())])))
}

/// Linha crua de `txn` (campos editáveis + vínculos).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TxnRow {
    pub id: Id,
    pub account_id: Id,
    pub date: String,
    pub amount: i64,
    pub currency: String,
    pub kind: TxnKind,
    pub status: TxnStatus,
    pub description: String,
    pub payee: Option<String>,
    pub notes: Option<String>,
    pub category_id: Option<Id>,
    pub member_id: Option<Id>,
    pub transfer_id: Option<Id>,
    pub statement_id: Option<Id>,
    pub installment_plan_id: Option<Id>,
    pub recurrence_id: Option<Id>,
}

const ROW_COLS: &str =
    "id, account_id, date, amount, currency, kind, status, description, payee, notes,
     category_id, member_id, transfer_id, statement_id, installment_plan_id, recurrence_id";

fn map_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<(TxnRow, String, String)> {
    Ok((
        TxnRow {
            id: r.get(0)?,
            account_id: r.get(1)?,
            date: r.get(2)?,
            amount: r.get(3)?,
            currency: r.get(4)?,
            kind: TxnKind::Income,
            status: TxnStatus::Cleared,
            description: r.get(7)?,
            payee: r.get(8)?,
            notes: r.get(9)?,
            category_id: r.get(10)?,
            member_id: r.get(11)?,
            transfer_id: r.get(12)?,
            statement_id: r.get(13)?,
            installment_plan_id: r.get(14)?,
            recurrence_id: r.get(15)?,
        },
        r.get(5)?,
        r.get(6)?,
    ))
}

fn finish((mut row, kind, status): (TxnRow, String, String)) -> Result<TxnRow> {
    row.kind = TxnKind::parse(&kind)?;
    row.status = TxnStatus::parse(&status)?;
    Ok(row)
}

pub(crate) fn load_row(conn: &Connection, id: &str) -> Result<TxnRow> {
    let raw = conn
        .query_row(
            &format!("SELECT {ROW_COLS} FROM txn WHERE id = ?1"),
            [id],
            map_row,
        )
        .optional()?
        .ok_or_else(|| Error::not_found(super::util::txn_ref(id)))?;
    finish(raw)
}

/// Outras pernas da mesma transferência.
pub(crate) fn other_legs(conn: &Connection, row: &TxnRow) -> Result<Vec<TxnRow>> {
    let Some(tid) = &row.transfer_id else {
        return Ok(vec![]);
    };
    let mut stmt = conn.prepare(&format!(
        "SELECT {ROW_COLS} FROM txn WHERE transfer_id = ?1 AND id <> ?2 ORDER BY id"
    ))?;
    let raw = stmt
        .query_map(params![tid, row.id], map_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    raw.into_iter().map(finish).collect()
}

/// Grava os campos editáveis de uma linha (só se algo mudou).
fn write_row(tx: &Transaction<'_>, before: &TxnRow, after: &TxnRow) -> Result<bool> {
    if before == after {
        return Ok(false);
    }
    tx.execute(
        "UPDATE txn SET account_id = ?2, date = ?3, amount = ?4, currency = ?5, kind = ?6, status = ?7,
                description = ?8, payee = ?9, notes = ?10, category_id = ?11, member_id = ?12,
                statement_id = ?13, fingerprint = ?14, updated_at = ?15
          WHERE id = ?1",
        params![
            after.id,
            after.account_id,
            after.date,
            after.amount,
            after.currency,
            after.kind.as_str(),
            after.status.as_str(),
            after.description,
            after.payee,
            after.notes,
            after.category_id,
            after.member_id,
            after.statement_id,
            fingerprint(&after.date, after.amount, &after.description),
            now_utc(),
        ],
    )?;
    Ok(true)
}

/// Categoria compatível com o tipo do lançamento: receita ↔ categoria de receita,
/// despesa ↔ categoria de despesa; os demais tipos não usam categoria.
pub(crate) fn check_category(
    conn: &Connection,
    category_id: Option<&str>,
    kind: TxnKind,
) -> Result<()> {
    let Some(cid) = category_id else {
        return Ok(());
    };
    let (ckind, archived, name) = crate::categories::kind_of(conn, cid)?;
    let expected = match kind {
        TxnKind::Income => CategoryKind::Income,
        TxnKind::Expense => CategoryKind::Expense,
        // Transferência também pode ser categorizada (ex.: PIX para terceiros); a de uma
        // perna só entra no fluxo de caixa pela categoria.
        TxnKind::Transfer => {
            if archived {
                return Err(invalid1(
                    "nucleo-lancamentos-erro-categoria-arquivada",
                    "nome",
                    name,
                ));
            }
            return Ok(());
        }
        other => {
            return Err(invalid1(
                "nucleo-lancamentos-erro-tipo-sem-categoria",
                "tipo",
                kind_label(other),
            ));
        }
    };
    if ckind != expected {
        let side = |income: bool| if income { "receita" } else { "despesa" };
        return Err(Error::invalid(t_args(
            "nucleo-lancamentos-erro-categoria-de-outro-tipo",
            &args(&[
                ("nome", name.into()),
                ("categoria", side(ckind == CategoryKind::Income).into()),
                ("lancamento", side(kind == TxnKind::Income).into()),
            ]),
        )));
    }
    if archived {
        return Err(invalid1(
            "nucleo-lancamentos-erro-categoria-arquivada",
            "nome",
            name,
        ));
    }
    Ok(())
}

fn check_splits(
    conn: &Connection,
    total: i64,
    kind: TxnKind,
    splits: &[SplitInput],
    currency: &str,
) -> Result<()> {
    if splits.is_empty() {
        return Ok(());
    }
    if !kind.is_cash_flow() {
        return Err(invalid1(
            "nucleo-lancamentos-erro-tipo-sem-divisao",
            "tipo",
            kind_label(kind),
        ));
    }
    validate_splits(total, splits, currency)?;
    for s in splits {
        check_category(conn, s.category_id.as_deref(), kind)?;
        if let Some(m) = &s.member_id {
            require_member(conn, m)?;
        }
    }
    Ok(())
}

fn existing_splits(conn: &Connection, txn_id: &str) -> Result<Vec<SplitInput>> {
    let mut stmt = conn.prepare(
        "SELECT category_id, member_id, amount, notes FROM txn_split WHERE txn_id = ?1 ORDER BY rowid",
    )?;
    let rows = stmt
        .query_map([txn_id], |r| {
            Ok(SplitInput {
                category_id: r.get(0)?,
                member_id: r.get(1)?,
                amount: r.get(2)?,
                notes: r.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

fn infer_kind(amount: i64) -> TxnKind {
    if amount < 0 {
        TxnKind::Expense
    } else {
        TxnKind::Income
    }
}

fn require_nonzero(amount: i64) -> Result<()> {
    if amount == 0 {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-valor-zero")));
    }
    Ok(())
}

/// Cria um lançamento simples (receita, despesa, ajuste ou aporte de uma perna só).
pub fn create(tx: &Transaction<'_>, input: &TxnInput, today: NaiveDate) -> Result<Id> {
    let (currency, _) = currency_and_kind(tx, &input.account_id)?;
    crate::time::parse_date(&input.date)?;
    require_nonzero(input.amount)?;
    let kind = input.kind.unwrap_or_else(|| infer_kind(input.amount));
    if kind == TxnKind::CardPayment {
        return Err(Error::invalid(t(
            "nucleo-lancamentos-erro-pagamento-duas-pernas",
        )));
    }
    let category_id = clean_opt(input.category_id.clone());
    check_category(tx, category_id.as_deref(), kind)?;
    let member_id = clean_opt(input.member_id.clone());
    if let Some(m) = &member_id {
        require_member(tx, m)?;
    }
    let splits = input.splits.clone().unwrap_or_default();
    check_splits(tx, input.amount, kind, &splits, &currency)?;
    let mut t = NewTxn::new(
        &input.account_id,
        &input.date,
        input.amount,
        &currency,
        kind,
        &input.description,
    );
    t.status = input.status.unwrap_or(TxnStatus::Cleared);
    t.payee = clean_opt(input.payee.clone());
    t.notes = clean_opt(input.notes.clone());
    t.category_id = category_id;
    t.member_id = member_id;
    t.card_plastic_id = clean_opt(input.card_plastic_id.clone());
    (t.orig_amount, t.orig_currency) = super::check_orig(
        input.amount,
        &currency,
        input.orig_amount,
        input.orig_currency.as_deref(),
    )?;
    if let Some(p) = &t.card_plastic_id {
        crate::cards::plastics::require_on_account(tx, p, &input.account_id)?;
    }
    let id = insert(tx, &t)?;
    if !splits.is_empty() {
        replace_splits(tx, &id, input.amount, &splits)?;
    }
    if let Some(tags) = &input.tags {
        replace_tags(tx, &id, tags)?;
    }
    if let Some(p) = clean_opt(input.project_id.clone()) {
        crate::projects::link(tx, std::slice::from_ref(&id), Some(&p))?;
    }
    let row = load_row(tx, &id)?;
    if let Some(s) = &row.statement_id {
        crate::cards::sync_statuses(tx, std::slice::from_ref(s), today)?;
    }
    Ok(id)
}

fn reconciled_error() -> Error {
    Error::conflict(t("nucleo-lancamentos-erro-conciliado"))
}

/// Novo status da outra perna quando uma perna muda de status.
fn leg_status(new: TxnStatus, other: TxnStatus) -> Result<TxnStatus> {
    Ok(match (new, other) {
        (TxnStatus::Planned, TxnStatus::Reconciled) => {
            return Err(Error::conflict(t(
                "nucleo-lancamentos-erro-outra-perna-conciliada-previsto",
            )));
        }
        (TxnStatus::Planned, _) => TxnStatus::Planned,
        (_, TxnStatus::Planned) => TxnStatus::Cleared,
        (_, o) => o,
    })
}

/// Fatura de uma linha numa conta de cartão (ou nenhuma, fora de cartão).
fn statement_for_row(tx: &Transaction<'_>, row: &TxnRow) -> Result<Option<Id>> {
    if row.kind == TxnKind::CardPayment && row.amount > 0 {
        crate::cards::statement_to_pay(tx, &row.account_id, &row.date)
    } else {
        crate::cards::statement_for(tx, &row.account_id, &row.date)
    }
}

/// Altera um lançamento. Pernas de transferência mantêm a outra perna coerente
/// (data, descrição, status e, na mesma moeda, o valor).
pub fn update(tx: &Transaction<'_>, patch: &TxnPatch, today: NaiveDate) -> Result<()> {
    let cur = load_row(tx, &patch.id)?;
    let mut new = cur.clone();
    if let Some(a) = &patch.account_id {
        new.account_id = a.clone();
    }
    if let Some(d) = &patch.date {
        crate::time::parse_date(d)?;
        new.date = d.clone();
    }
    if let Some(a) = patch.amount {
        require_nonzero(a)?;
        new.amount = a;
    }
    if let Some(k) = patch.kind {
        new.kind = k;
    }
    if let Some(s) = patch.status {
        new.status = s;
    }
    if let Some(d) = &patch.description {
        new.description = d.trim().to_string();
    }
    if let Some(p) = &patch.payee {
        new.payee = clean_opt(p.clone());
    }
    if let Some(n) = &patch.notes {
        new.notes = clean_opt(n.clone());
    }
    if let Some(c) = &patch.category_id {
        new.category_id = clean_opt(c.clone());
    }
    if let Some(m) = &patch.member_id {
        new.member_id = clean_opt(m.clone());
        if let Some(m) = &new.member_id {
            require_member(tx, m)?;
        }
    }
    let core_changed =
        new.account_id != cur.account_id || new.date != cur.date || new.amount != cur.amount;
    if cur.status == TxnStatus::Reconciled && new.status == TxnStatus::Reconciled && core_changed {
        return Err(reconciled_error());
    }
    let mut touched: BTreeSet<Id> = cur.statement_id.iter().cloned().collect();

    if new.account_id != cur.account_id {
        let (cur_new, akind) = currency_and_kind(tx, &new.account_id)?;
        if cur_new != cur.currency {
            return Err(Error::invalid(t_args(
                "nucleo-lancamentos-erro-conta-outra-moeda",
                &args(&[
                    ("moeda_conta", cur_new.into()),
                    ("moeda", cur.currency.clone().into()),
                ]),
            )));
        }
        if cur.kind == TxnKind::CardPayment && cur.amount > 0 && akind != AccountKind::CreditCard {
            return Err(Error::invalid(t(
                "nucleo-lancamentos-erro-pagamento-destino-cartao",
            )));
        }
    }

    let legs = other_legs(tx, &cur)?;
    if cur.transfer_id.is_some() {
        if new.kind != cur.kind {
            return Err(Error::invalid(t(
                "nucleo-lancamentos-erro-perna-mudar-tipo",
            )));
        }
        if new.amount.signum() != cur.amount.signum() {
            return Err(Error::invalid(t("nucleo-lancamentos-erro-perna-sinal")));
        }
        if legs.iter().any(|l| l.account_id == new.account_id) {
            return Err(Error::invalid(t(
                "nucleo-lancamentos-erro-pernas-mesma-conta",
            )));
        }
        if patch.splits.as_ref().is_some_and(|s| !s.is_empty()) {
            return Err(Error::invalid(t(
                "nucleo-lancamentos-erro-transferencia-dividida",
            )));
        }
    } else if new.kind == TxnKind::CardPayment && new.kind != cur.kind {
        return Err(Error::invalid(t(
            "nucleo-lancamentos-erro-virar-pagamento-fatura",
        )));
    }
    if patch.transfer_account_id.is_some() && cur.transfer_id.is_some() {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-ja-ligado")));
    }

    if new.category_id != cur.category_id || new.kind != cur.kind {
        check_category(tx, new.category_id.as_deref(), new.kind)?;
    }
    // Divisão: a enviada, ou a existente conferida contra o novo valor/tipo.
    match &patch.splits {
        Some(s) => {
            check_splits(tx, new.amount, new.kind, s, &cur.currency)?;
            replace_splits(tx, &cur.id, new.amount, s)?;
        }
        None if new.amount != cur.amount || new.kind != cur.kind => {
            let existing = existing_splits(tx, &cur.id)?;
            if !existing.is_empty() {
                check_splits(tx, new.amount, new.kind, &existing, &cur.currency).map_err(|e| {
                    Error::invalid(t_args(
                        "nucleo-lancamentos-erro-divisao-desatualizada",
                        &args(&[("erro", e.message().into())]),
                    ))
                })?;
            }
        }
        None => {}
    }
    if new.account_id != cur.account_id
        || (new.date != cur.date && cur.kind != TxnKind::CardPayment)
    {
        new.statement_id = statement_for_row(tx, &new)?;
    }
    touched.extend(new.statement_id.iter().cloned());
    write_row(tx, &cur, &new)?;
    // Valor de origem (compra no exterior): o enviado, conferido contra o valor novo.
    if patch.orig_amount.is_some() || patch.orig_currency.is_some() || new.amount != cur.amount {
        let (old_amount, old_cur): (Option<i64>, Option<String>) = tx.query_row(
            "SELECT orig_amount, orig_currency FROM txn WHERE id = ?1",
            [&cur.id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let amount = match patch.orig_amount {
            Some(v) => v,
            None => old_amount,
        };
        let currency = match (&patch.orig_amount, &patch.orig_currency) {
            (Some(None), _) => None,
            (_, Some(c)) => Some(c.clone()),
            _ => old_cur,
        };
        let (a, c) = super::check_orig(new.amount, &cur.currency, amount, currency.as_deref())?;
        tx.execute(
            "UPDATE txn SET orig_amount = ?2, orig_currency = ?3 WHERE id = ?1
                AND (orig_amount IS NOT ?2 OR orig_currency IS NOT ?3)",
            params![cur.id, a, c],
        )?;
    }
    // Plástico do cartão: o enviado (do cartão novo) ou, ao mudar de conta, nenhum.
    let plastic = match &patch.card_plastic_id {
        Some(p) => Some(clean_opt(p.clone())),
        None if new.account_id != cur.account_id => Some(None),
        None => None,
    };
    if let Some(p) = &plastic {
        if let Some(p) = p {
            crate::cards::plastics::require_on_account(tx, p, &new.account_id)?;
        }
        tx.execute(
            "UPDATE txn SET card_plastic_id = ?2 WHERE id = ?1 AND card_plastic_id IS NOT ?2",
            params![cur.id, p],
        )?;
    }
    if patch.member_id.is_some() {
        mark_member_manual(tx, &cur.id)?;
    } else if new.account_id != cur.account_id || plastic.is_some() {
        // Mudou de conta ou de plástico com o membro herdado: passa a ser o do plástico ou o
        // da conta nova.
        super::reinherit_one(tx, &cur.id)?;
    }
    // Mudou o tipo à mão: o vínculo com a operação (que exige aporte) deixa de valer.
    if new.kind != TxnKind::Investment {
        tx.execute("DELETE FROM txn_link WHERE txn_id = ?1", [&cur.id])?;
    }
    if let Some(tags) = &patch.tags {
        replace_tags(tx, &cur.id, tags)?;
    }
    if let Some(p) = &patch.project_id {
        crate::projects::link(
            tx,
            std::slice::from_ref(&cur.id),
            clean_opt(p.clone()).as_deref(),
        )?;
    }

    // Outras pernas da transferência.
    for leg in legs {
        let mut l = leg.clone();
        if new.date != cur.date {
            l.date = new.date.clone();
        }
        if patch.description.is_some() {
            l.description = new.description.clone();
        }
        if patch.member_id.is_some() {
            l.member_id = new.member_id.clone();
        }
        if new.amount != cur.amount && l.currency == cur.currency {
            l.amount = -new.amount;
        }
        if new.status != cur.status {
            l.status = leg_status(new.status, l.status)?;
        }
        let leg_core = l.date != leg.date || l.amount != leg.amount;
        if leg.status == TxnStatus::Reconciled && l.status == TxnStatus::Reconciled && leg_core {
            return Err(Error::conflict(t(
                "nucleo-lancamentos-erro-outra-perna-conciliada",
            )));
        }
        if l.date != leg.date && l.kind != TxnKind::CardPayment {
            l.statement_id = statement_for_row(tx, &l)?;
        }
        touched.extend(leg.statement_id.iter().cloned());
        touched.extend(l.statement_id.iter().cloned());
        write_row(tx, &leg, &l)?;
    }
    let touched: Vec<Id> = touched.into_iter().collect();
    crate::cards::sync_statuses(tx, &touched, today)?;
    if let Some(acc) = &patch.transfer_account_id {
        super::transfers::link_to_account(tx, &cur.id, acc, patch.transfer_amount, today)?;
    }
    Ok(())
}

/// Apaga lançamentos (e a outra perna das transferências), com divisões, tags e anexos.
/// Devolve quantas linhas foram apagadas.
pub fn delete(tx: &Transaction<'_>, ids: &[Id], today: NaiveDate) -> Result<usize> {
    delete_checked(tx, ids, today, true)
}

/// Como [`delete`]; `refuse_reconciled: false` apaga também os conciliados (cópias entre
/// contas: quem chama refaz os marcos de saldo que contavam com elas).
pub(crate) fn delete_checked(
    tx: &Transaction<'_>,
    ids: &[Id],
    today: NaiveDate,
    refuse_reconciled: bool,
) -> Result<usize> {
    let mut all: Vec<TxnRow> = Vec::new();
    let mut seen = BTreeSet::new();
    for id in dedup_ids(ids) {
        let row = load_row(tx, &id)?;
        let legs = other_legs(tx, &row)?;
        for r in std::iter::once(row).chain(legs) {
            if seen.insert(r.id.clone()) {
                all.push(r);
            }
        }
    }
    if let Some(r) = all
        .iter()
        .find(|r| refuse_reconciled && r.status == TxnStatus::Reconciled)
    {
        return Err(Error::conflict(t_args(
            "nucleo-lancamentos-erro-apagar-conciliado",
            &args(&[
                ("descricao", r.description.clone().into()),
                ("data", super::util::date_text(&r.date).into()),
            ]),
        )));
    }
    delete_rows(tx, &all, today)
}

/// Apaga as linhas indicadas sem checagens (uso interno: parcelamentos, recorrências).
pub(crate) fn delete_rows(
    tx: &Transaction<'_>,
    rows: &[TxnRow],
    today: NaiveDate,
) -> Result<usize> {
    let mut touched = BTreeSet::new();
    // Parcelas de dívida pagas por estes lançamentos voltam a ficar em aberto.
    let ids: Vec<Id> = rows.iter().map(|r| r.id.clone()).collect();
    crate::debt::link::forget_txns(tx, &ids)?;
    for r in rows {
        tx.execute("DELETE FROM txn_tag WHERE txn_id = ?1", [&r.id])?;
        tx.execute("DELETE FROM txn_split WHERE txn_id = ?1", [&r.id])?;
        tx.execute("DELETE FROM attachment WHERE txn_id = ?1", [&r.id])?;
        tx.execute("DELETE FROM txn WHERE id = ?1", [&r.id])?;
        touched.extend(r.statement_id.iter().cloned());
    }
    let touched: Vec<Id> = touched.into_iter().collect();
    crate::cards::sync_statuses(tx, &touched, today)?;
    Ok(rows.len())
}

/// Ids criados por uma transferência.
#[derive(Debug, Clone)]
pub struct TransferIds {
    pub transfer_id: Id,
    pub from_id: Id,
    pub to_id: Id,
}

/// Cria as duas pernas de uma transferência (ou pagamento de fatura, ou aporte).
/// `statement_id` força a fatura da perna do cartão num pagamento.
pub fn create_transfer_with(
    tx: &Transaction<'_>,
    input: &TransferInput,
    statement_id: Option<Id>,
    today: NaiveDate,
) -> Result<TransferIds> {
    if input.from_account_id == input.to_account_id {
        return Err(Error::invalid(t(
            "nucleo-lancamentos-erro-transferencia-mesma-conta",
        )));
    }
    let (from_cur, from_kind) = currency_and_kind(tx, &input.from_account_id)?;
    let (to_cur, to_kind) = currency_and_kind(tx, &input.to_account_id)?;
    crate::time::parse_date(&input.date)?;
    if input.amount <= 0 {
        return Err(Error::invalid(t(
            "nucleo-lancamentos-erro-transferencia-valor",
        )));
    }
    let to_amount = if from_cur == to_cur {
        match input.to_amount {
            Some(v) if v != input.amount => {
                return Err(Error::invalid(t(
                    "nucleo-lancamentos-erro-transferencia-mesma-moeda",
                )));
            }
            _ => input.amount,
        }
    } else {
        match input.to_amount {
            Some(v) if v > 0 => v,
            _ => {
                return Err(Error::invalid(t_args(
                    "nucleo-lancamentos-erro-transferencia-outra-moeda",
                    &args(&[
                        ("moeda_origem", from_cur.clone().into()),
                        ("moeda_destino", to_cur.clone().into()),
                    ]),
                )));
            }
        }
    };
    let kind = input.kind.unwrap_or(TxnKind::Transfer);
    match kind {
        TxnKind::Transfer | TxnKind::Investment => {}
        TxnKind::CardPayment => {
            if to_kind != AccountKind::CreditCard {
                return Err(Error::invalid(t(
                    "nucleo-lancamentos-erro-pagamento-so-cartao",
                )));
            }
            if from_kind == AccountKind::CreditCard {
                return Err(Error::invalid(t(
                    "nucleo-lancamentos-erro-pagamento-de-cartao",
                )));
            }
        }
        other => {
            return Err(invalid1(
                "nucleo-lancamentos-erro-transferencia-tipo",
                "tipo",
                kind_label(other),
            ));
        }
    }
    let member_id = clean_opt(input.member_id.clone());
    if let Some(m) = &member_id {
        require_member(tx, m)?;
    }
    let name = |id: &str| -> Result<String> {
        Ok(tx.query_row("SELECT name FROM account WHERE id = ?1", [id], |r| r.get(0))?)
    };
    let (from_name, to_name) = (name(&input.from_account_id)?, name(&input.to_account_id)?);
    let custom = clean_opt(input.description.clone());
    // Descrição padrão no idioma da instalação (fica gravada como a do usuário).
    let named =
        |key: &str, account: &str| t_args(key, &args(&[("conta", account.to_string().into())]));
    let (from_desc, to_desc) = match (&custom, kind) {
        (Some(d), _) => (d.clone(), d.clone()),
        (None, TxnKind::CardPayment) => (
            named("nucleo-lancamentos-descricao-pagamento-fatura", &to_name),
            named(
                "nucleo-lancamentos-descricao-pagamento-recebido",
                &from_name,
            ),
        ),
        (None, TxnKind::Investment) => (
            named("nucleo-lancamentos-descricao-aporte-em", &to_name),
            named("nucleo-lancamentos-descricao-aporte-vindo", &from_name),
        ),
        (None, _) => (
            named("nucleo-lancamentos-descricao-transferencia-para", &to_name),
            named("nucleo-lancamentos-descricao-transferencia-de", &from_name),
        ),
    };
    let status = input.status.unwrap_or(TxnStatus::Cleared);
    let transfer_id = new_id();
    let mut from = NewTxn::new(
        &input.from_account_id,
        &input.date,
        -input.amount,
        &from_cur,
        kind,
        &from_desc,
    );
    from.status = status;
    from.member_id = member_id.clone();
    from.transfer_id = Some(transfer_id.clone());
    let mut to = NewTxn::new(
        &input.to_account_id,
        &input.date,
        to_amount,
        &to_cur,
        kind,
        &to_desc,
    );
    to.status = status;
    to.member_id = member_id;
    to.transfer_id = Some(transfer_id.clone());
    if kind == TxnKind::CardPayment {
        to.statement_id = match statement_id {
            Some(s) => Some(s),
            None => crate::cards::statement_to_pay(tx, &input.to_account_id, &input.date)?,
        };
    }
    let from_id = insert(tx, &from)?;
    let to_id = insert(tx, &to)?;
    let touched: Vec<Id> = [&from_id, &to_id]
        .into_iter()
        .filter_map(|id| {
            tx.query_row("SELECT statement_id FROM txn WHERE id = ?1", [id], |r| {
                r.get::<_, Option<String>>(0)
            })
            .ok()
            .flatten()
        })
        .collect();
    crate::cards::sync_statuses(tx, &touched, today)?;
    Ok(TransferIds {
        transfer_id,
        from_id,
        to_id,
    })
}

pub fn create_transfer(
    tx: &Transaction<'_>,
    input: &TransferInput,
    today: NaiveDate,
) -> Result<TransferIds> {
    create_transfer_with(tx, input, None, today)
}

/// Define (ou remove, com `None`) a categoria de vários lançamentos. Lançamentos
/// divididos perdem a divisão: a categoria escolhida passa a valer para o total.
/// Aportes, pagamentos de fatura e ajustes não usam categoria (ficam fora do fluxo de caixa):
/// numa seleção misturada eles ficam de fora e a contagem volta em `skipped`; só eles, erro.
/// Devolve (alterados, deixados de fora).
pub fn set_category(
    tx: &Transaction<'_>,
    ids: &[Id],
    category_id: Option<&str>,
) -> Result<(usize, usize)> {
    let ids = dedup_ids(ids);
    let mut rows = ids
        .iter()
        .map(|id| load_row(tx, id))
        .collect::<Result<Vec<_>>>()?;
    let mut skipped = 0;
    if let Some(cid) = category_id {
        let before = rows.len();
        rows.retain(|r| r.kind.is_cash_flow() || r.kind == TxnKind::Transfer);
        skipped = before - rows.len();
        if rows.is_empty() {
            return Err(Error::invalid(t_args(
                "nucleo-lancamentos-erro-lote-sem-categoria",
                &args(&[("n", skipped.into())]),
            )));
        }
        for k in rows
            .iter()
            .map(|r| r.kind)
            .collect::<std::collections::HashSet<_>>()
        {
            check_category(tx, Some(cid), k)?;
        }
    }
    let mut n = 0;
    for r in rows {
        let had_splits = tx.execute("DELETE FROM txn_split WHERE txn_id = ?1", [&r.id])? > 0;
        let mut new = r.clone();
        new.category_id = category_id.map(str::to_string);
        if write_row(tx, &r, &new)? || had_splits {
            n += 1;
        }
    }
    Ok((n, skipped))
}

/// Define (ou remove, com `None`) o titular de vários lançamentos; as outras pernas das
/// transferências acompanham, como na edição. Devolve quantos lançamentos pedidos mudaram.
/// Membro escolhido à mão (inclusive "família", sem membro): a conta não muda mais ele.
fn mark_member_manual(tx: &Transaction<'_>, id: &str) -> Result<()> {
    tx.execute(
        "UPDATE txn SET member_origin = 'manual' WHERE id = ?1 AND member_origin IS NOT 'manual'",
        [id],
    )?;
    Ok(())
}

pub fn set_member(tx: &Transaction<'_>, ids: &[Id], member_id: Option<&str>) -> Result<usize> {
    if let Some(m) = member_id {
        require_member(tx, m)?;
    }
    let mut n = 0;
    for id in dedup_ids(ids) {
        let cur = load_row(tx, &id)?;
        if cur.member_id.as_deref() == member_id {
            continue;
        }
        let mut new = cur.clone();
        new.member_id = member_id.map(str::to_string);
        write_row(tx, &cur, &new)?;
        mark_member_manual(tx, &cur.id)?;
        n += 1;
        for leg in other_legs(tx, &cur)? {
            let mut l = leg.clone();
            l.member_id = new.member_id.clone();
            write_row(tx, &leg, &l)?;
            mark_member_manual(tx, &leg.id)?;
        }
    }
    Ok(n)
}

/// Muda o status de vários lançamentos (pernas de transferência acompanham:
/// previsto ↔ efetivado). Devolve quantos lançamentos pedidos mudaram.
pub fn set_status(
    tx: &Transaction<'_>,
    ids: &[Id],
    status: TxnStatus,
    today: NaiveDate,
) -> Result<usize> {
    let mut n = 0;
    let mut touched = BTreeSet::new();
    for id in dedup_ids(ids) {
        let cur = load_row(tx, &id)?;
        if cur.status == status {
            continue;
        }
        let mut new = cur.clone();
        new.status = status;
        write_row(tx, &cur, &new)?;
        touched.extend(cur.statement_id.iter().cloned());
        n += 1;
        for leg in other_legs(tx, &cur)? {
            let mut l = leg.clone();
            l.status = leg_status(status, leg.status)?;
            touched.extend(leg.statement_id.iter().cloned());
            write_row(tx, &leg, &l)?;
        }
    }
    let touched: Vec<Id> = touched.into_iter().collect();
    crate::cards::sync_statuses(tx, &touched, today)?;
    Ok(n)
}
