//! Operação de câmbio entre contas próprias em moedas diferentes (ex.: real para dólar numa
//! corretora no exterior): a transferência com o valor que sai e o que entra, a taxa
//! aplicada, a de referência e os encargos (spread, IOF, tarifa), que viram despesas.
//!
//! "Moeda da casa" = a moeda base da família (o real no Brasil, o dólar nos EUA…).
//!
//! Referência: no Brasil, a PTAX do Banco Central (oficial e gratuita): o fechamento do dia,
//! quando já coletado, ou o último boletim do dia (abertura e intermediários) pela API Olinda;
//! fora do Brasil, o câmbio já coletado (BCE e outras fontes). O usuário sempre pode digitar a
//! taxa e os encargos que o banco cobrou.
//!
//! Taxa: "1 unidade estrangeira = X da moeda da casa" quando um lado é a moeda da casa; entre
//! duas moedas estrangeiras, "1 unidade de origem = X de destino". O IOF (imposto brasileiro)
//! é sobre o valor na moeda da casa e cai na conta dessa moeda.

use std::str::FromStr;
use std::time::Duration;

use chrono::NaiveDate;
use rusqlite::{Connection, Transaction, params};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{self, args, t_args};
use crate::ids::{Id, new_id};
use crate::money::minor_units;
use crate::sources::SourceId;
use crate::time::{fmt_date, now_utc, parse_date};
use crate::transactions::{NewTxn, TransferInput, TxnKind};

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct ReferenceRateArgs {
    /// Moeda estrangeira (ex.: USD).
    pub currency: String,
    /// Padrão: hoje.
    #[serde(default)]
    #[ts(optional)]
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReferenceRate {
    pub currency: String,
    /// 1 unidade = `sell` da moeda base da família (no Brasil, PTAX venda) e `buy` (PTAX compra).
    pub sell: String,
    pub buy: Option<String>,
    pub date: String,
    /// `Fechamento`, `Abertura`, `Intermediário` ou `coletada` (fechamento já no cofre).
    pub bulletin: String,
    pub source: String,
    /// De outro dia (sem boletim na data pedida).
    pub stale: bool,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct ExchangeArgs {
    pub from_account_id: Id,
    pub to_account_id: Id,
    pub date: String,
    /// Valor que sai da origem, sem encargos (na moeda dela). Informe este ou `amount_to`.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub amount_from: Option<i64>,
    /// Valor que entra no destino (na moeda dele).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub amount_to: Option<i64>,
    /// Taxa aplicada pelo banco ("1 estrangeira = X da moeda base"). Sem ela: referência ×
    /// (1 + spread).
    #[serde(default)]
    #[ts(optional)]
    pub rate: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub reference_rate: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub reference_source: Option<String>,
    /// Spread sobre a referência, em %.
    #[serde(default)]
    #[ts(optional)]
    pub spread_pct: Option<String>,
    /// IOF em % sobre o valor na moeda base (no Brasil, em reais).
    #[serde(default)]
    #[ts(optional)]
    pub iof_pct: Option<String>,
    /// Tarifa fixa, na moeda da origem.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub fee: Option<i64>,
    #[serde(default)]
    #[ts(optional)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExchangeQuote {
    pub from_currency: String,
    pub to_currency: String,
    #[ts(type = "number")]
    pub amount_from: i64,
    #[ts(type = "number")]
    pub amount_to: i64,
    pub rate: String,
    pub spread_pct: Option<String>,
    #[ts(type = "number")]
    pub iof_amount: i64,
    #[ts(type = "number")]
    pub fee_amount: i64,
    /// O que sai da origem no total (valor + IOF + tarifa, quando estão na moeda da origem).
    #[ts(type = "number")]
    pub total_from: i64,
    /// Valor efetivo total: moeda base por unidade estrangeira, com todos os encargos.
    pub vet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FxOperation {
    pub id: Id,
    pub transfer_id: Id,
    pub date: String,
    pub from_account_id: Id,
    pub to_account_id: Id,
    pub from_currency: String,
    pub to_currency: String,
    #[ts(type = "number")]
    pub amount_from: i64,
    #[ts(type = "number")]
    pub amount_to: i64,
    pub rate: String,
    pub reference_rate: Option<String>,
    pub reference_source: Option<String>,
    pub spread_pct: Option<String>,
    pub iof_pct: Option<String>,
    #[ts(type = "number")]
    pub iof_amount: i64,
    #[ts(type = "number")]
    pub fee_amount: i64,
    pub vet: Option<String>,
    pub notes: Option<String>,
}

/// `what`: chave do rótulo do campo (`nucleo-contas-cambio-campo-*`), para a mensagem.
fn dec(s: &str, what: &str) -> Result<Decimal> {
    Decimal::from_str(s.trim().replace(',', ".").as_str()).map_err(|_| {
        Error::invalid(t_args(
            "nucleo-contas-cambio-erro-numero",
            &args(&[
                ("campo", i18n::t(what).into()),
                ("valor", s.to_string().into()),
            ]),
        ))
    })
}

fn opt_dec(s: &Option<String>, what: &str) -> Result<Option<Decimal>> {
    match s.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok(None),
        Some(v) => dec(v, what).map(Some),
    }
}

fn to_units(minor: i64, ccy: &str) -> Decimal {
    Decimal::from(minor) / Decimal::from(10i64.pow(minor_units(ccy)))
}

/// Teto de um valor (10 trilhões na moeda): acima disso é erro de digitação ou de taxa.
const MAX_MINOR: i64 = 1_000_000_000_000_000;

fn to_minor(units: Decimal, ccy: &str) -> Result<i64> {
    (units * Decimal::from(10i64.pow(minor_units(ccy))))
        .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
        .to_i64()
        .filter(|v| v.abs() <= MAX_MINOR)
        .ok_or_else(|| Error::invalid(i18n::t("nucleo-contas-cambio-erro-grande")))
}

/// Calcula valores, IOF, tarifa e VET (sem gravar nada).
pub fn quote(conn: &Connection, a: &ExchangeArgs) -> Result<ExchangeQuote> {
    let (from_ccy, _) = crate::accounts::currency_and_kind(conn, &a.from_account_id)?;
    let (to_ccy, _) = crate::accounts::currency_and_kind(conn, &a.to_account_id)?;
    if from_ccy == to_ccy {
        return Err(Error::invalid(i18n::t(
            "nucleo-contas-cambio-erro-mesma-moeda",
        )));
    }
    let reference = opt_dec(&a.reference_rate, "nucleo-contas-cambio-campo-referencia")?;
    let spread = opt_dec(&a.spread_pct, "nucleo-contas-cambio-campo-spread")?;
    let rate = match (
        opt_dec(&a.rate, "nucleo-contas-cambio-campo-taxa")?,
        reference,
    ) {
        (Some(r), _) => r,
        (None, Some(r)) => r * (Decimal::ONE + spread.unwrap_or_default() / Decimal::ONE_HUNDRED),
        (None, None) => {
            return Err(Error::invalid(i18n::t(
                "nucleo-contas-cambio-erro-sem-taxa",
            )));
        }
    };
    if rate <= Decimal::ZERO {
        return Err(Error::invalid(i18n::t(
            "nucleo-contas-cambio-erro-taxa-positiva",
        )));
    }
    // `rate` = moeda da casa por estrangeira; entre estrangeiras, destino por origem.
    let home = crate::household::base_currency(conn)?;
    let home_from = from_ccy == home;
    let home_to = to_ccy == home;
    let convert_from_to = |from_units: Decimal| -> Decimal {
        if home_from {
            from_units / rate
        } else {
            from_units * rate
        }
    };
    let convert_to_from = |to_units: Decimal| -> Decimal {
        if home_from {
            to_units * rate
        } else {
            to_units / rate
        }
    };
    let (amount_from, amount_to) = match (a.amount_from, a.amount_to) {
        (Some(f), _) if f > 0 => (
            f,
            to_minor(convert_from_to(to_units(f, &from_ccy)), &to_ccy)?,
        ),
        (_, Some(t)) if t > 0 => (
            to_minor(convert_to_from(to_units(t, &to_ccy)), &from_ccy)?,
            t,
        ),
        _ => {
            return Err(Error::invalid(i18n::t("nucleo-contas-cambio-erro-valor")));
        }
    };
    let iof_pct = opt_dec(&a.iof_pct, "nucleo-contas-cambio-campo-iof")?;
    // IOF sobre o valor na moeda da casa, cobrado na conta dessa moeda (a origem, numa remessa).
    let iof_amount = match iof_pct {
        Some(p) if p > Decimal::ZERO => {
            let base_home = if home_from {
                to_units(amount_from, &from_ccy)
            } else if home_to {
                to_units(amount_to, &to_ccy)
            } else {
                Decimal::ZERO
            };
            to_minor(base_home * p / Decimal::ONE_HUNDRED, &home)?
        }
        _ => 0,
    };
    let fee = a.fee.unwrap_or(0).max(0);
    let total_from = amount_from + fee + if home_from { iof_amount } else { 0 };
    let vet = if home_from {
        Some(
            (to_units(total_from, &from_ccy) / to_units(amount_to, &to_ccy))
                .round_dp(6)
                .normalize()
                .to_string(),
        )
    } else if home_to {
        Some(
            ((to_units(amount_to, &to_ccy) - to_units(iof_amount, &home))
                / to_units(amount_from + fee, &from_ccy))
            .round_dp(6)
            .normalize()
            .to_string(),
        )
    } else {
        None
    };
    Ok(ExchangeQuote {
        from_currency: from_ccy,
        to_currency: to_ccy,
        amount_from,
        amount_to,
        rate: rate.round_dp(8).normalize().to_string(),
        spread_pct: match (reference, a.rate.as_deref()) {
            (Some(r), Some(_)) if r > Decimal::ZERO => Some(
                ((rate / r - Decimal::ONE) * Decimal::ONE_HUNDRED)
                    .round_dp(4)
                    .normalize()
                    .to_string(),
            ),
            _ => spread.map(|s| s.normalize().to_string()),
        },
        iof_amount,
        fee_amount: fee,
        total_from,
        vet,
    })
}

/// Categoria de despesa pelo nome em pt-BR (vale com as categorias em qualquer idioma).
fn category_named(conn: &Connection, child: &str) -> Option<String> {
    let want = child.to_lowercase();
    crate::categories::br_categories(conn)
        .ok()?
        .into_iter()
        .find(|c| {
            c.kind == crate::categories::CategoryKind::Expense
                && !c.archived
                && c.name.to_lowercase() == want
        })
        .map(|c| c.id)
}

/// Grava a transferência, os encargos (IOF e tarifa como despesas) e a operação.
pub fn execute(tx: &Transaction<'_>, a: &ExchangeArgs, today: NaiveDate) -> Result<Id> {
    let q = quote(tx, a)?;
    let date = fmt_date(parse_date(&a.date)?);
    let (from_name, to_name) = (
        crate::accounts::account_name(tx, &a.from_account_id)?,
        crate::accounts::account_name(tx, &a.to_account_id)?,
    );
    let ids = crate::transactions::create_transfer_with(
        tx,
        &TransferInput {
            from_account_id: a.from_account_id.clone(),
            to_account_id: a.to_account_id.clone(),
            date: date.clone(),
            amount: q.amount_from,
            to_amount: Some(q.amount_to),
            // Descrições e notas no idioma da instalação (ficam gravadas nos lançamentos).
            description: Some(t_args(
                "nucleo-contas-cambio-descricao",
                &args(&[
                    ("origem", q.from_currency.clone().into()),
                    ("destino", q.to_currency.clone().into()),
                ]),
            )),
            kind: None,
            status: None,
            member_id: None,
        },
        None,
        today,
    )?;
    let _ = (from_name, to_name);
    let mut fees: Vec<Id> = vec![];
    // IOF fica na conta da moeda da casa; tarifa, na origem.
    if q.iof_amount > 0 {
        let home = crate::household::base_currency(tx)?;
        let (acc, ccy) = if q.from_currency == home {
            (&a.from_account_id, &q.from_currency)
        } else {
            (&a.to_account_id, &q.to_currency)
        };
        let mut t = NewTxn::new(
            acc,
            &date,
            -q.iof_amount,
            ccy,
            TxnKind::Expense,
            &i18n::t("nucleo-contas-cambio-descricao-iof"),
        );
        t.category_id = category_named(tx, "IOF");
        t.notes = Some(t_args(
            "nucleo-contas-cambio-nota-iof",
            &args(&[
                ("origem", q.from_currency.clone().into()),
                ("destino", q.to_currency.clone().into()),
                ("pct", a.iof_pct.clone().unwrap_or_default().into()),
            ]),
        ));
        fees.push(crate::transactions::insert(tx, &t)?);
    }
    if q.fee_amount > 0 {
        let mut t = NewTxn::new(
            &a.from_account_id,
            &date,
            -q.fee_amount,
            &q.from_currency,
            TxnKind::Expense,
            &i18n::t("nucleo-contas-cambio-descricao-tarifa"),
        );
        t.category_id = category_named(tx, "Tarifas bancárias");
        fees.push(crate::transactions::insert(tx, &t)?);
    }
    let id = new_id();
    let now = now_utc();
    tx.execute(
        "INSERT INTO fx_operation (id, transfer_id, date, from_account_id, to_account_id, from_currency, to_currency,
                                   amount_from, amount_to, rate, reference_rate, reference_source, spread_pct, iof_pct,
                                   iof_amount, fee_amount, fees_txn_ids, vet, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?20)",
        params![
            id,
            ids.transfer_id,
            date,
            a.from_account_id,
            a.to_account_id,
            q.from_currency,
            q.to_currency,
            q.amount_from,
            q.amount_to,
            q.rate,
            a.reference_rate,
            a.reference_source,
            q.spread_pct,
            a.iof_pct,
            q.iof_amount,
            q.fee_amount,
            serde_json::to_string(&fees)?,
            q.vet,
            a.notes,
            now
        ],
    )?;
    // A taxa aplicada também serve de câmbio do dia para o patrimônio (manual, se não houver PTAX).
    Ok(id)
}

pub fn list(conn: &Connection, account_id: Option<&str>) -> Result<Vec<FxOperation>> {
    let mut stmt = conn.prepare(
        "SELECT id, transfer_id, date, from_account_id, to_account_id, from_currency, to_currency, amount_from,
                amount_to, rate, reference_rate, reference_source, spread_pct, iof_pct, iof_amount, fee_amount, vet, notes
           FROM fx_operation
          WHERE (?1 IS NULL OR from_account_id = ?1 OR to_account_id = ?1)
            AND EXISTS (SELECT 1 FROM txn t WHERE t.transfer_id = fx_operation.transfer_id)
          ORDER BY date DESC, created_at DESC",
    )?;
    Ok(stmt
        .query_map([account_id], |r| {
            Ok(FxOperation {
                id: r.get(0)?,
                transfer_id: r.get(1)?,
                date: r.get(2)?,
                from_account_id: r.get(3)?,
                to_account_id: r.get(4)?,
                from_currency: r.get(5)?,
                to_currency: r.get(6)?,
                amount_from: r.get(7)?,
                amount_to: r.get(8)?,
                rate: r.get(9)?,
                reference_rate: r.get(10)?,
                reference_source: r.get(11)?,
                spread_pct: r.get(12)?,
                iof_pct: r.get(13)?,
                iof_amount: r.get(14)?,
                fee_amount: r.get(15)?,
                vet: r.get(16)?,
                notes: r.get(17)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?)
}

/// Último boletim PTAX do dia (abertura, intermediários ou fechamento): `(hora, tipo, compra, venda)`.
pub fn parse_bulletins(body: &str) -> Result<Option<(String, String, Decimal, Decimal)>> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Row {
        cotacao_compra: serde_json::Number,
        cotacao_venda: serde_json::Number,
        data_hora_cotacao: String,
        tipo_boletim: Option<String>,
    }
    #[derive(Deserialize)]
    struct Resp {
        value: Vec<Row>,
    }
    let r: Resp = serde_json::from_str(body).map_err(|e| {
        Error::invalid(t_args(
            "nucleo-contas-cambio-erro-ptax",
            &args(&[("erro", e.to_string().into())]),
        ))
    })?;
    let Some(last) = r
        .value
        .into_iter()
        .max_by(|a, b| a.data_hora_cotacao.cmp(&b.data_hora_cotacao))
    else {
        return Ok(None);
    };
    Ok(Some((
        last.data_hora_cotacao,
        last.tipo_boletim.unwrap_or_else(|| "Fechamento".into()),
        dec(
            &last.cotacao_compra.to_string(),
            "nucleo-contas-cambio-campo-ptax",
        )?,
        dec(
            &last.cotacao_venda.to_string(),
            "nucleo-contas-cambio-campo-ptax",
        )?,
    )))
}

/// Nome do boletim PTAX do BCB (`tipoBoletim`: Abertura, Intermediário, Fechamento) no idioma
/// da instalação; outro valor passa como veio.
fn bulletin_label(kind: &str) -> String {
    match kind {
        "Abertura" => i18n::t("nucleo-contas-ptax-abertura"),
        "Intermediário" => i18n::t("nucleo-contas-ptax-intermediario"),
        "Fechamento" => i18n::t("nucleo-contas-ptax-fechamento"),
        other => other.to_string(),
    }
}

/// Referência do dia na moeda base da família: fechamento já coletado; senão, com a moeda base
/// em reais, o último boletim do BCB (rede, fora do banco); senão o último câmbio coletado
/// antes da data.
pub fn reference_rate(api: &crate::Api, a: &ReferenceRateArgs) -> Result<ReferenceRate> {
    let ccy = a.currency.trim().to_uppercase();
    crate::money::validate_currency(&ccy)?;
    let date = match &a.date {
        Some(d) => parse_date(d)?,
        None => crate::time::today_local(),
    };
    let ds = fmt_date(date);
    let (home, stored, bcb_on) = api.with_session(|s, _| {
        let c = s.db.conn();
        let home = crate::household::base_currency(c)?;
        if ccy == home {
            return Ok((home, None, false));
        }
        Ok((
            home.clone(),
            super::rate(c, &ccy, &home, &ds)?,
            // A PTAX é sempre em reais.
            home == "BRL" && crate::sources::enabled(c, SourceId::Bcb),
        ))
    })?;
    if ccy == home {
        return Err(Error::invalid(i18n::t(
            "nucleo-contas-cambio-erro-referencia-brl",
        )));
    }
    if let Some((r, d, src, _)) = &stored
        && *d == ds
    {
        return Ok(ReferenceRate {
            currency: ccy,
            sell: r.normalize().to_string(),
            buy: None,
            date: d.clone(),
            bulletin: "coletada".into(),
            source: src.clone(),
            stale: false,
        });
    }
    if bcb_on {
        let url = format!(
            "https://olinda.bcb.gov.br/olinda/servico/PTAX/versao/v1/odata/CotacaoMoedaDia(moeda=@moeda,dataCotacao=@dataCotacao)?@moeda='{ccy}'&@dataCotacao='{}'&$format=json&$select=cotacaoCompra,cotacaoVenda,dataHoraCotacao,tipoBoletim",
            date.format("%m-%d-%Y")
        );
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .build()
            .into();
        let body = agent
            .get(&url)
            .header("User-Agent", "Treasury (gestão financeira local)")
            .call()
            .ok()
            .and_then(|mut r| r.body_mut().read_to_string().ok());
        if let Some(b) = body
            && let Ok(Some((when, kind, buy, sell))) = parse_bulletins(&b)
        {
            return Ok(ReferenceRate {
                currency: ccy,
                sell: sell.normalize().to_string(),
                buy: Some(buy.normalize().to_string()),
                date: when.get(..10).unwrap_or(&ds).to_string(),
                bulletin: bulletin_label(&kind),
                source: "bcb_ptax".into(),
                stale: false,
            });
        }
    }
    match stored {
        Some((r, d, src, _)) => Ok(ReferenceRate {
            currency: ccy,
            sell: r.normalize().to_string(),
            buy: None,
            date: d,
            bulletin: "coletada".into(),
            source: src,
            stale: true,
        }),
        None => Err(Error::not_found(t_args(
            "nucleo-contas-cambio-sem-referencia",
            &args(&[("moeda", ccy.into())]),
        ))),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::api::Api;
    use crate::paths::TreasuryPaths;

    #[test]
    fn brl_to_usd_with_spread_iof_and_fee() {
        let dir = tempfile::tempdir().unwrap();
        let api =
            Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
                .unwrap();
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
        )
        .unwrap();
        let acc = |name: &str, kind: &str, ccy: &str| {
            api.call(
                "accounts.create",
                json!({"name": name, "kind": kind, "currency": ccy, "opening_date": "2026-01-01"}),
            )
            .unwrap()["id"]
                .as_str()
                .unwrap()
                .to_string()
        };
        let brl = acc("Itaú", "checking", "BRL");
        let usd = acc("Avenue", "foreign", "USD");
        let args = json!({"from_account_id": brl, "to_account_id": usd, "date": "2026-10-10",
                          "amount_to": 100_000, "reference_rate": "5,40", "spread_pct": "1",
                          "iof_pct": "1,1", "fee": 1_500, "reference_source": "bcb_ptax"});
        let q = api.call("fx.exchange_quote", args.clone()).unwrap();
        // US$ 1,000 × 5.454 = R$ 5,454.00; IOF 1.1% = R$ 59.99; fee R$ 15.
        assert_eq!(q["rate"], "5.454");
        assert_eq!(q["amount_from"], 545_400);
        assert_eq!(q["iof_amount"], 5_999);
        assert_eq!(q["total_from"], 545_400 + 5_999 + 1_500);
        assert_eq!(q["vet"], "5.52899");
        let op = api.call("fx.exchange", args).unwrap();
        assert_eq!(op["amount_to"], 100_000);
        let list = api.call("fx.operations", json!({})).unwrap();
        assert_eq!(list.as_array().unwrap().len(), 1);
        // The amount + IOF + fee went out; US$ 1,000 came in.
        let b = api.call("accounts.get", json!({"id": brl})).unwrap();
        assert_eq!(b["balance"], -(545_400 + 5_999 + 1_500));
        let u = api.call("accounts.get", json!({"id": usd})).unwrap();
        assert_eq!(u["balance"], 100_000);
        // Same currency is a transfer, not an exchange.
        assert!(
            api.call(
                "fx.exchange_quote",
                json!({"from_account_id": brl, "to_account_id": brl, "date": "2026-10-10",
                       "amount_from": 1, "rate": "1"})
            )
            .is_err()
        );
        let p = super::parse_bulletins(
            r#"{"value":[{"cotacaoCompra":5.2,"cotacaoVenda":5.21,"dataHoraCotacao":"2026-10-16 10:10:11.5","tipoBoletim":"Abertura"},
                         {"cotacaoCompra":5.3,"cotacaoVenda":5.31,"dataHoraCotacao":"2026-10-16 11:09:00.1","tipoBoletim":"Intermediário"}]}"#,
        )
        .unwrap()
        .unwrap();
        assert_eq!(p.1, "Intermediário");
        assert_eq!(p.3.to_string(), "5.31");
    }
}
