//! Gravação de operações, eventos corporativos, proventos, alvos e watchlist. Compras,
//! vendas e proventos movimentam o caixa da conta da corretora (lançamento ligado).

#![allow(clippy::type_complexity)] // linhas lidas como tuplas do SQLite

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Row, Transaction, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

use super::*;
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};
use crate::ids::new_id;
use crate::money::minor_units;
use crate::time::{now_utc, parse_date};
use crate::transactions::{self, NewTxn, TxnKind};

/// Decimal de um campo; `what` é a chave do nome do campo (`nucleo-carteira-campo-…`) ou o
/// próprio nome.
pub fn dec(s: &str, what: &str) -> Result<Decimal> {
    s.trim()
        .replace(',', ".")
        .parse::<Decimal>()
        .map_err(|_| invalid_field(what, s))
}

/// "<campo> inválido: <valor>"; `what` é a chave do nome do campo ou o próprio nome.
pub(crate) fn invalid_field(what: &str, value: &str) -> Error {
    Error::invalid(t_args(
        "nucleo-carteira-erro-campo-invalido",
        &i18n::args(&[
            ("campo", t(what).into()),
            ("valor", value.to_string().into()),
        ]),
    ))
}

/// Decimal no formato do idioma da instalação (vírgula ou ponto).
pub(crate) fn fmt_dec(v: Decimal) -> String {
    crate::ai::alerts::decimal(i18n::current(), &v.normalize().to_string())
}

/// Texto do catálogo com argumentos de texto.
pub(crate) fn text(key: &str, list: &[(&'static str, String)]) -> String {
    let mut a = i18n::Args::new();
    for (k, v) in list {
        a.insert(std::borrow::Cow::Borrowed(*k), v.clone().into());
    }
    t_args(key, &a)
}

/// "operação <id>", para o erro de não encontrado.
fn trade_ref(id: &str) -> String {
    t_args(
        "nucleo-carteira-operacao",
        &i18n::args(&[("id", id.to_string().into())]),
    )
}

/// Teto de um valor em menor unidade (10 trilhões na moeda): saturar aqui deixa espaço para
/// somas sem estourar o `i64`.
pub const MAX_MINOR: i64 = 1_000_000_000_000_000;

/// Decimal na moeda → menor unidade, arredondando metade para longe do zero (limitado a
/// ±[`MAX_MINOR`]).
pub fn to_minor(v: Decimal, currency: &str) -> i64 {
    (v * Decimal::from(10i64.pow(minor_units(currency))))
        .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
        .to_i64()
        .unwrap_or(if v.is_sign_negative() {
            -MAX_MINOR
        } else {
            MAX_MINOR
        })
        .clamp(-MAX_MINOR, MAX_MINOR)
}

pub fn from_minor(v: i64, currency: &str) -> Decimal {
    Decimal::from(v) / Decimal::from(10i64.pow(minor_units(currency)))
}

pub(crate) fn fmt_qty(q: Decimal) -> String {
    fmt_dec(q)
}

fn brief(conn: &Connection, asset_id: &str) -> Result<AssetBrief> {
    let a = crate::assets::get(conn, asset_id)?;
    Ok(AssetBrief {
        id: a.id,
        ticker: a.ticker,
        name: a.name,
        class: a.class,
        class_label: a.class_label,
        currency: a.currency,
    })
}

pub fn asset_brief(conn: &Connection, asset_id: &str) -> Result<AssetBrief> {
    brief(conn, asset_id)
}

fn row_to_trade(r: &Row<'_>) -> rusqlite::Result<Trade> {
    let side: String = r.get("side")?;
    let qty: String = r.get("quantity")?;
    let price: String = r.get("price")?;
    let currency: String = r.get("currency")?;
    let fees: i64 = r.get("fees")?;
    let taxes: i64 = r.get("taxes")?;
    let gross = match (qty.parse::<Decimal>(), price.parse::<Decimal>()) {
        (Ok(q), Ok(p)) => to_minor(q * p, &currency),
        _ => 0,
    };
    let side = if side == "sell" {
        TradeSide::Sell
    } else {
        TradeSide::Buy
    };
    let cash_amount = match side {
        TradeSide::Buy => -(gross + fees + taxes),
        TradeSide::Sell => gross - fees - taxes,
    };
    Ok(Trade {
        id: r.get("id")?,
        asset_id: r.get("asset_id")?,
        ticker: r.get("ticker")?,
        asset_name: r.get("asset_name")?,
        by_value: r.get::<_, String>("asset_class")? == "fixed_income",
        account_id: r.get("account_id")?,
        account_name: r.get("account_name")?,
        date: r.get("date")?,
        side,
        quantity: qty,
        price,
        fees,
        taxes,
        currency,
        fx_rate: r.get("fx_rate")?,
        day_trade: r.get::<_, i64>("day_trade")? == 1,
        gross,
        cash_amount,
        txn_id: r.get("txn_id")?,
        note_number: r.get("note_number")?,
        notes: r.get("notes")?,
        import_id: r.get("import_id")?,
    })
}

const TRADE_SELECT: &str = "SELECT t.*, a.ticker, a.name AS asset_name, a.class AS asset_class,
    ac.name AS account_name
    FROM trade t JOIN asset a ON a.id = t.asset_id JOIN account ac ON ac.id = t.account_id";

pub fn get_trade(conn: &Connection, id: &str) -> Result<Trade> {
    conn.query_row(
        &format!("{TRADE_SELECT} WHERE t.id = ?1"),
        [id],
        row_to_trade,
    )
    .optional()?
    .ok_or_else(|| Error::not_found(trade_ref(id)))
}

pub fn list_trades(conn: &Connection, f: &TradeFilter) -> Result<Vec<Trade>> {
    let sql = format!(
        "{TRADE_SELECT} WHERE (?1 IS NULL OR t.asset_id = ?1) AND (?2 IS NULL OR t.account_id = ?2)
           AND (?3 IS NULL OR t.date >= ?3) AND (?4 IS NULL OR t.date <= ?4)
         ORDER BY t.date DESC, t.created_at DESC LIMIT ?5 OFFSET ?6"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(
            params![
                f.asset_id,
                f.account_id,
                f.date_from,
                f.date_to,
                f.limit.unwrap_or(500).clamp(1, 5000),
                f.offset.unwrap_or(0)
            ],
            row_to_trade,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Valor do caixa da operação na moeda da conta.
fn cash_in_account(
    account_currency: &str,
    asset_currency: &str,
    amount: i64,
    fx: Option<Decimal>,
) -> Result<i64> {
    if account_currency == asset_currency {
        return Ok(amount);
    }
    match (account_currency, fx) {
        (acc, Some(rate)) => Ok(to_minor(from_minor(amount, asset_currency) * rate, acc)),
        _ => Err(Error::invalid(t_args(
            "nucleo-carteira-erro-moedas",
            &i18n::args(&[
                ("conta", account_currency.to_string().into()),
                ("ativo", asset_currency.to_string().into()),
            ]),
        ))),
    }
}

fn validate_trade(
    conn: &Connection,
    t: &TradeInput,
) -> Result<(crate::assets::Asset, Decimal, Decimal, Option<Decimal>)> {
    parse_date(&t.date)?;
    let asset = crate::assets::get(conn, &t.asset_id)?;
    crate::accounts::currency_and_kind(conn, &t.account_id)?;
    let qty = dec(&t.quantity, "nucleo-carteira-campo-quantidade")?;
    let price = dec(&t.price, "nucleo-carteira-campo-preco")?;
    if qty <= Decimal::ZERO {
        return Err(Error::invalid(i18n::t("nucleo-carteira-erro-quantidade")));
    }
    if price < Decimal::ZERO {
        return Err(Error::invalid(i18n::t("nucleo-carteira-erro-preco")));
    }
    if t.fees.unwrap_or(0) < 0 || t.taxes.unwrap_or(0) < 0 {
        return Err(Error::invalid(i18n::t("nucleo-carteira-erro-custos")));
    }
    let fx = t
        .fx_rate
        .as_deref()
        .map(|s| dec(s, "nucleo-carteira-campo-cambio"))
        .transpose()?;
    if let Some(f) = fx
        && f <= Decimal::ZERO
    {
        return Err(Error::invalid(i18n::t("nucleo-carteira-erro-cambio")));
    }
    Ok((asset, qty, price, fx))
}

fn trade_description(
    side: TradeSide,
    qty: Decimal,
    asset: &crate::assets::Asset,
    price: Decimal,
) -> String {
    let key = if side == TradeSide::Buy {
        "nucleo-carteira-operacao-compra"
    } else {
        "nucleo-carteira-operacao-venda"
    };
    let label = asset.ticker.clone().unwrap_or_else(|| asset.name.clone());
    t_args(
        key,
        &i18n::args(&[
            ("quantidade", fmt_qty(qty).into()),
            ("ativo", label.into()),
            (
                "preco",
                i18n::money(
                    i18n::current(),
                    to_minor(price, &asset.currency),
                    &asset.currency,
                )
                .into(),
            ),
        ]),
    )
}

pub fn create_trade(tx: &Transaction<'_>, t: &TradeInput) -> Result<Id> {
    create_trade_with(tx, t, false)
}

/// Como [`create_trade`]; com `allow_oversell`, grava a venda mesmo maior que a posição
/// na data (importação de notas: a compra pode ter sido em outra corretora ou ainda não
/// importada; a carteira limita a venda à posição e avisa).
pub fn create_trade_with(tx: &Transaction<'_>, t: &TradeInput, allow_oversell: bool) -> Result<Id> {
    let (asset, qty, price, fx) = validate_trade(tx, t)?;
    if t.side == TradeSide::Sell && !allow_oversell {
        // Não permite vender mais do que a posição na conta até a data.
        let held = super::engine::quantity_held(tx, &t.asset_id, Some(&t.account_id), &t.date)?;
        if qty > held {
            return Err(Error::invalid(t_args(
                "nucleo-carteira-erro-venda-maior",
                &i18n::args(&[
                    ("quantidade", fmt_qty(qty).into()),
                    ("posicao", fmt_qty(held).into()),
                ]),
            )));
        }
    }
    let id = new_id();
    let now = now_utc();
    let fees = t.fees.unwrap_or(0);
    let taxes = t.taxes.unwrap_or(0);
    let gross = to_minor(qty * price, &asset.currency);
    let cash = match t.side {
        TradeSide::Buy => -(gross + fees + taxes),
        TradeSide::Sell => gross - fees - taxes,
    };
    let txn_id = if t.cash_movement.unwrap_or(true) {
        let (acc_cur, _) = crate::accounts::currency_and_kind(tx, &t.account_id)?;
        let fx_for_cash = fx.or_else(|| {
            crate::fx::rate(tx, &asset.currency, &acc_cur, &t.date)
                .ok()
                .flatten()
                .map(|(r, ..)| r)
        });
        let amount = cash_in_account(&acc_cur, &asset.currency, cash, fx_for_cash)?;
        let mut n = NewTxn::new(
            &t.account_id,
            &t.date,
            amount,
            &acc_cur,
            TxnKind::Investment,
            &trade_description(t.side, qty, &asset, price),
        );
        n.notes = t.notes.clone();
        Some(transactions::insert(tx, &n)?)
    } else {
        None
    };
    tx.execute(
        "INSERT INTO trade (id, asset_id, account_id, date, side, quantity, price, fees, taxes, currency, fx_rate, day_trade,
                            txn_id, note_number, external_id, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?17)",
        params![
            id,
            t.asset_id,
            t.account_id,
            t.date,
            if t.side == TradeSide::Buy { "buy" } else { "sell" },
            qty.normalize().to_string(),
            price.normalize().to_string(),
            fees,
            taxes,
            asset.currency,
            fx.map(|f| f.normalize().to_string()),
            t.day_trade.unwrap_or(false) as i64,
            txn_id,
            t.note_number,
            t.external_id,
            t.notes,
            now
        ],
    )?;
    Ok(id)
}

pub fn update_trade(tx: &Transaction<'_>, p: &TradePatch) -> Result<()> {
    let cur = get_trade(tx, &p.id)?;
    let input = TradeInput {
        asset_id: cur.asset_id.clone(),
        account_id: cur.account_id.clone(),
        date: p.date.clone().unwrap_or(cur.date.clone()),
        side: p.side.unwrap_or(cur.side),
        quantity: p.quantity.clone().unwrap_or(cur.quantity.clone()),
        price: p.price.clone().unwrap_or(cur.price.clone()),
        fees: Some(p.fees.unwrap_or(cur.fees)),
        taxes: Some(p.taxes.unwrap_or(cur.taxes)),
        fx_rate: p.fx_rate.clone().unwrap_or(cur.fx_rate.clone()),
        day_trade: Some(p.day_trade.unwrap_or(cur.day_trade)),
        note_number: p.note_number.clone().unwrap_or(cur.note_number.clone()),
        notes: p.notes.clone().unwrap_or(cur.notes.clone()),
        cash_movement: Some(p.cash_movement.unwrap_or(cur.txn_id.is_some())),
        external_id: None,
    };
    // Regrava a operação (e o lançamento de caixa ligado) com os novos valores.
    delete_trade(tx, &p.id)?;
    let new_id = create_trade(tx, &input)?;
    tx.execute(
        "UPDATE trade SET id = ?1, import_id = ?3 WHERE id = ?2",
        params![p.id, new_id, cur.import_id],
    )?;
    Ok(())
}

pub fn delete_trade(tx: &Transaction<'_>, id: &str) -> Result<()> {
    let txn_id: Option<String> = tx
        .query_row("SELECT txn_id FROM trade WHERE id = ?1", [id], |r| r.get(0))
        .optional()?
        .ok_or_else(|| Error::not_found(trade_ref(id)))?;
    tx.execute("DELETE FROM trade WHERE id = ?1", [id])?;
    if let Some(t) = txn_id {
        tx.execute("DELETE FROM txn WHERE id = ?1", [t])?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Eventos corporativos

fn kind_str<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_value(v)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}

fn parse_kind<T: serde::de::DeserializeOwned>(s: &str) -> rusqlite::Result<T> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}

fn row_to_action(r: &Row<'_>) -> rusqlite::Result<CorporateAction> {
    Ok(CorporateAction {
        id: r.get("id")?,
        asset_id: r.get("asset_id")?,
        date: r.get("date")?,
        kind: parse_kind(&r.get::<_, String>("kind")?)?,
        factor: r.get("factor")?,
        unit_cost: r.get("unit_cost")?,
        amount: r.get("amount")?,
        new_asset_id: r.get("new_asset_id")?,
        notes: r.get("notes")?,
        source: r.get("source")?,
    })
}

pub fn list_actions(conn: &Connection, asset_id: Option<&str>) -> Result<Vec<CorporateAction>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM corporate_action WHERE (?1 IS NULL OR asset_id = ?1) ORDER BY date",
    )?;
    Ok(stmt
        .query_map([asset_id], row_to_action)?
        .collect::<rusqlite::Result<_>>()?)
}

pub fn create_action(tx: &Transaction<'_>, a: &CorporateActionInput) -> Result<Id> {
    parse_date(&a.date)?;
    crate::assets::get(tx, &a.asset_id)?;
    let factor = a
        .factor
        .as_deref()
        .map(|f| dec(f, "nucleo-carteira-campo-fator"))
        .transpose()?;
    match a.kind {
        CorporateActionKind::Split if factor.is_none_or(|f| f <= Decimal::ONE) => {
            return Err(Error::invalid(t("nucleo-carteira-erro-desdobramento")));
        }
        CorporateActionKind::ReverseSplit
            if factor.is_none_or(|f| f <= Decimal::ZERO || f >= Decimal::ONE) =>
        {
            return Err(Error::invalid(t("nucleo-carteira-erro-grupamento")));
        }
        CorporateActionKind::Bonus if factor.is_none_or(|f| f <= Decimal::ZERO) => {
            return Err(Error::invalid(t("nucleo-carteira-erro-bonificacao")));
        }
        CorporateActionKind::CapitalReturn if a.amount.is_none_or(|v| v <= 0) => {
            return Err(Error::invalid(t("nucleo-carteira-erro-devolucao")));
        }
        CorporateActionKind::TickerChange | CorporateActionKind::Merger
            if a.new_asset_id.is_none() =>
        {
            return Err(Error::invalid(t("nucleo-carteira-erro-ativo-resultante")));
        }
        _ => {}
    }
    let id = new_id();
    let now = now_utc();
    tx.execute(
        "INSERT INTO corporate_action (id, asset_id, date, kind, factor, unit_cost, amount, new_asset_id, notes, source, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'manual', ?10, ?10)",
        params![
            id,
            a.asset_id,
            a.date,
            kind_str(&a.kind),
            factor.map(|f| f.normalize().to_string()),
            a.unit_cost.as_deref().map(|u| dec(u, "custo")).transpose()?.map(|u| u.normalize().to_string()),
            a.amount,
            a.new_asset_id,
            a.notes,
            now
        ],
    )?;
    Ok(id)
}

pub fn delete_action(tx: &Transaction<'_>, id: &str) -> Result<()> {
    if tx.execute("DELETE FROM corporate_action WHERE id = ?1", [id])? == 0 {
        return Err(Error::not_found(format!("evento {id}")));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Proventos

fn row_to_income(r: &Row<'_>) -> rusqlite::Result<IncomeEvent> {
    Ok(IncomeEvent {
        id: r.get("id")?,
        asset_id: r.get("asset_id")?,
        ticker: r.get("ticker")?,
        asset_name: r.get("asset_name")?,
        account_id: r.get("account_id")?,
        kind: parse_kind(&r.get::<_, String>("kind")?)?,
        ex_date: r.get("ex_date")?,
        pay_date: r.get("pay_date")?,
        quantity: r.get("quantity")?,
        gross: r.get("gross")?,
        tax_withheld: r.get("tax_withheld")?,
        net: r.get("net")?,
        currency: r.get("currency")?,
        fx_rate: r.get("fx_rate")?,
        status: if r.get::<_, String>("status")? == "announced" {
            IncomeStatus::Announced
        } else {
            IncomeStatus::Received
        },
        txn_id: r.get("txn_id")?,
        source: r.get("source")?,
    })
}

pub fn list_income(
    conn: &Connection,
    asset_id: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Vec<IncomeEvent>> {
    let mut stmt = conn.prepare(
        "SELECT i.*, a.ticker, a.name AS asset_name FROM income_event i JOIN asset a ON a.id = i.asset_id
         WHERE (?1 IS NULL OR i.asset_id = ?1) AND (?2 IS NULL OR i.pay_date >= ?2) AND (?3 IS NULL OR i.pay_date <= ?3)
         ORDER BY i.pay_date DESC",
    )?;
    Ok(stmt
        .query_map(params![asset_id, from, to], row_to_income)?
        .collect::<rusqlite::Result<_>>()?)
}

/// Categoria de receita para o provento (Rendimentos › Dividendos/JCP), se existir.
fn income_category(conn: &Connection, kind: IncomeKind) -> Result<Option<Id>> {
    let child = match kind {
        IncomeKind::Jcp => "JCP",
        IncomeKind::Interest | IncomeKind::Coupon => "Juros",
        IncomeKind::Rent => "Aluguéis recebidos",
        _ => "Dividendos",
    };
    // Pelos nomes em pt-BR: vale com as categorias em qualquer idioma.
    crate::categories::find_br_path(
        conn,
        Some(crate::categories::CategoryKind::Income),
        &format!("Rendimentos{}{child}", crate::categories::PATH_SEP),
    )
}

/// Retenção do JCP na data do crédito ou do pagamento (a que vier primeiro): 15% até 2025 e
/// 17,5% a partir de 01/01/2026.
///
/// Fontes: LC 224/2025 art. 8º (nova redação da Lei 9.249/1995 art. 9º §2º: "17,5% ... na data
/// do pagamento ou do crédito ao beneficiário") e art. 14, III (vigência em 1º/01/2026);
/// Receita, Nota Cetad/Coest nº 009/2026 (a alíquota "passou a ser fixada em 17,5%"). Não há
/// regra de transição: JCP de 2025 creditado ou pago em 2026 já sai com 17,5%.
pub fn jcp_rate(on: NaiveDate) -> Decimal {
    if on >= NaiveDate::from_ymd_opt(2026, 1, 1).expect("data") {
        Decimal::new(175, 3)
    } else {
        Decimal::new(15, 2)
    }
}

/// Data que decide a retenção do JCP: a do crédito (aproximada pela data-com) ou a do
/// pagamento, a que vier primeiro.
pub fn credit_date(ex: Option<NaiveDate>, pay: NaiveDate) -> NaiveDate {
    ex.map_or(pay, |e| e.min(pay))
}

/// IR retido na fonte presumido: JCP pela alíquota da data (`on`, ver [`jcp_rate`]) e 30% em
/// dividendos de ativos americanos.
pub fn default_withholding(
    kind: IncomeKind,
    class: crate::assets::AssetClass,
    gross: i64,
    on: NaiveDate,
) -> i64 {
    use crate::assets::AssetClass as C;
    let rate = match (kind, class) {
        (IncomeKind::Jcp, _) => jcp_rate(on),
        (IncomeKind::Dividend, C::StockUs | C::EtfUs | C::Reit) => Decimal::new(30, 2),
        // Outras bolsas: a retenção depende do país e do tratado; o usuário informa.
        (IncomeKind::Dividend, C::StockIntl | C::EtfIntl) => return 0,
        _ => return 0,
    };
    (Decimal::from(gross) * rate).round().to_i64().unwrap_or(0)
}

pub fn create_income(tx: &Transaction<'_>, i: &IncomeInput) -> Result<Id> {
    parse_date(&i.pay_date)?;
    if let Some(d) = &i.ex_date {
        parse_date(d)?;
    }
    let asset = crate::assets::get(tx, &i.asset_id)?;
    if i.gross <= 0 {
        return Err(Error::invalid(t("nucleo-carteira-erro-bruto")));
    }
    let tax = i.tax_withheld.unwrap_or_else(|| {
        let ex = i.ex_date.as_deref().and_then(|d| parse_date(d).ok());
        let on = credit_date(ex, parse_date(&i.pay_date).unwrap_or_default());
        default_withholding(i.kind, asset.class, i.gross, on)
    });
    if tax < 0 || tax > i.gross {
        return Err(Error::invalid(t("nucleo-carteira-erro-imposto-retido")));
    }
    let net = i.gross - tax;
    let status = i.status.unwrap_or(IncomeStatus::Received);
    let fx = i
        .fx_rate
        .as_deref()
        .map(|s| dec(s, "nucleo-carteira-campo-cambio"))
        .transpose()?;
    let txn_id = if status == IncomeStatus::Received && i.cash_movement.unwrap_or(true) {
        let (acc_cur, _) = crate::accounts::currency_and_kind(tx, &i.account_id)?;
        let fx_for_cash = fx.or_else(|| {
            crate::fx::rate(tx, &asset.currency, &acc_cur, &i.pay_date)
                .ok()
                .flatten()
                .map(|(r, ..)| r)
        });
        let amount = cash_in_account(&acc_cur, &asset.currency, net, fx_for_cash)?;
        let label = asset.ticker.clone().unwrap_or_else(|| asset.name.clone());
        let mut n = NewTxn::new(
            &i.account_id,
            &i.pay_date,
            amount,
            &acc_cur,
            TxnKind::Income,
            &t_args(
                "nucleo-carteira-provento-lancamento",
                &i18n::args(&[("tipo", i.kind.label().into()), ("ativo", label.into())]),
            ),
        );
        n.category_id = income_category(tx, i.kind)?;
        Some(transactions::insert(tx, &n)?)
    } else {
        None
    };
    let id = new_id();
    let now = now_utc();
    tx.execute(
        "INSERT INTO income_event (id, asset_id, account_id, kind, ex_date, pay_date, quantity, gross, tax_withheld, net, currency,
                                   fx_rate, status, txn_id, source, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 'manual', ?15, ?15)",
        params![
            id,
            i.asset_id,
            i.account_id,
            kind_str(&i.kind),
            i.ex_date,
            i.pay_date,
            i.quantity.as_deref().map(|q| dec(q, "quantidade")).transpose()?.map(|q| q.normalize().to_string()),
            i.gross,
            tax,
            net,
            asset.currency,
            fx.map(|f| f.normalize().to_string()),
            if status == IncomeStatus::Announced { "announced" } else { "received" },
            txn_id,
            now
        ],
    )?;
    Ok(id)
}

pub fn delete_income(tx: &Transaction<'_>, id: &str) -> Result<()> {
    let txn_id: Option<String> = tx
        .query_row("SELECT txn_id FROM income_event WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .optional()?
        .ok_or_else(|| Error::not_found(format!("provento {id}")))?;
    tx.execute("DELETE FROM income_event WHERE id = ?1", [id])?;
    if let Some(t) = txn_id {
        tx.execute("DELETE FROM txn WHERE id = ?1", [t])?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Alvos e watchlist

pub fn list_targets(conn: &Connection) -> Result<Vec<Target>> {
    let mut stmt = conn.prepare("SELECT id, scope, key, target_pct, tolerance_pct FROM target_allocation ORDER BY scope, key")?;
    let rows: Vec<(String, String, String, String, String)> = stmt
        .query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    rows.into_iter()
        .map(|(id, scope, key, t, tol)| {
            let scope = if scope == "asset" {
                TargetScope::Asset
            } else {
                TargetScope::Class
            };
            let label = match scope {
                TargetScope::Class => crate::assets::AssetClass::parse(&key)
                    .map(|c| c.label())
                    .unwrap_or(key.clone()),
                TargetScope::Asset => crate::assets::get(conn, &key)
                    .map(|a| a.ticker.unwrap_or(a.name))
                    .unwrap_or(key.clone()),
            };
            Ok(Target {
                id,
                scope,
                key,
                label,
                target_pct: t,
                tolerance_pct: tol,
            })
        })
        .collect()
}

/// Substitui todos os alvos de um escopo.
pub fn set_targets(
    tx: &Transaction<'_>,
    scope: TargetScope,
    targets: &[TargetInput],
) -> Result<()> {
    let scope_s = if scope == TargetScope::Asset {
        "asset"
    } else {
        "class"
    };
    let mut total = Decimal::ZERO;
    for t in targets {
        if t.scope != scope {
            return Err(Error::invalid(i18n::t("nucleo-carteira-erro-alvos-escopo")));
        }
        let pct = dec(&t.target_pct, "nucleo-carteira-campo-alvo")?;
        if pct < Decimal::ZERO || pct > Decimal::ONE_HUNDRED {
            return Err(Error::invalid(i18n::t("nucleo-carteira-erro-alvo")));
        }
        total += pct;
        if scope == TargetScope::Class {
            crate::assets::AssetClass::parse(&t.key)?;
        } else {
            crate::assets::get(tx, &t.key)?;
        }
    }
    if scope == TargetScope::Class && total > Decimal::ONE_HUNDRED {
        return Err(Error::invalid(i18n::t("nucleo-carteira-erro-alvos-soma")));
    }
    if scope == TargetScope::Asset {
        // Alvo por ativo é a fatia dentro da classe dele.
        let mut by_class: std::collections::HashMap<crate::assets::AssetClass, Decimal> =
            Default::default();
        for t in targets {
            let class = crate::assets::get(tx, &t.key)?.class;
            *by_class.entry(class).or_default() +=
                dec(&t.target_pct, "nucleo-carteira-campo-alvo")?;
        }
        if let Some((c, _)) = by_class.iter().find(|(_, v)| **v > Decimal::ONE_HUNDRED) {
            return Err(Error::invalid(t_args(
                "nucleo-carteira-erro-alvos-soma-classe",
                &i18n::args(&[("classe", c.label().into())]),
            )));
        }
    }
    tx.execute("DELETE FROM target_allocation WHERE scope = ?1", [scope_s])?;
    let now = now_utc();
    for t in targets {
        let tol = t
            .tolerance_pct
            .as_deref()
            .map(|s| dec(s, "nucleo-carteira-campo-tolerancia"))
            .transpose()?
            .unwrap_or(Decimal::from(5));
        tx.execute(
            "INSERT INTO target_allocation (id, scope, key, target_pct, tolerance_pct, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![new_id(), scope_s, t.key, dec(&t.target_pct, "nucleo-carteira-campo-alvo")?.normalize().to_string(), tol.normalize().to_string(), now],
        )?;
    }
    Ok(())
}

pub fn upsert_watch(tx: &Transaction<'_>, w: &WatchlistInput) -> Result<Id> {
    crate::assets::get(tx, &w.asset_id)?;
    let norm = |s: &Option<String>, what: &str| -> Result<Option<String>> {
        s.as_deref()
            .map(|v| dec(v, what).map(|d| d.normalize().to_string()))
            .transpose()
    };
    let now = now_utc();
    let existing: Option<String> = tx
        .query_row(
            "SELECT id FROM watchlist_item WHERE asset_id = ?1",
            [&w.asset_id],
            |r| r.get(0),
        )
        .optional()?;
    let id = existing.clone().unwrap_or_else(new_id);
    tx.execute(
        "INSERT INTO watchlist_item (id, asset_id, target_price, alert_above, alert_below, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
         ON CONFLICT (asset_id) DO UPDATE SET target_price = excluded.target_price, alert_above = excluded.alert_above,
             alert_below = excluded.alert_below, notes = excluded.notes, updated_at = excluded.updated_at",
        params![
            id,
            w.asset_id,
            norm(&w.target_price, "nucleo-carteira-campo-preco-alvo")?,
            norm(&w.alert_above, "nucleo-carteira-campo-alerta")?,
            norm(&w.alert_below, "nucleo-carteira-campo-alerta")?,
            w.notes,
            now
        ],
    )?;
    Ok(id)
}

pub fn list_watch(conn: &Connection) -> Result<Vec<WatchlistItem>> {
    let mut stmt =
        conn.prepare("SELECT id, asset_id, target_price, alert_above, alert_below, notes FROM watchlist_item ORDER BY created_at")?;
    let rows: Vec<(
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = stmt
        .query_map([], |r| {
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
    let today = crate::time::today_local();
    rows.into_iter()
        .map(|(id, asset_id, target, above, below, notes)| {
            let price = crate::market::price_on(conn, &asset_id, today)?;
            let p = price.as_ref().map(|(v, _)| *v);
            let parse = |s: &Option<String>| s.as_deref().and_then(|v| v.parse::<Decimal>().ok());
            let upside = match (p, parse(&target)) {
                (Some(p), Some(t)) if !p.is_zero() => (t / p - Decimal::ONE).to_f64(),
                _ => None,
            };
            let mut triggered = vec![];
            if let (Some(p), Some(a)) = (p, parse(&above))
                && p >= a
            {
                triggered.push(t_args(
                    "nucleo-carteira-watchlist-acima",
                    &i18n::args(&[("valor", fmt_dec(a).into())]),
                ));
            }
            if let (Some(p), Some(b)) = (p, parse(&below))
                && p <= b
            {
                triggered.push(t_args(
                    "nucleo-carteira-watchlist-abaixo",
                    &i18n::args(&[("valor", fmt_dec(b).into())]),
                ));
            }
            Ok(WatchlistItem {
                id,
                asset: brief(conn, &asset_id)?,
                target_price: target,
                alert_above: above,
                alert_below: below,
                notes,
                price: price.map(|(_, p)| p),
                upside_to_target: upside,
                triggered,
            })
        })
        .collect()
}
