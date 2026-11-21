//! Cadastro de investimentos em lote (`portfolio.bulk`), para o agente que lê uma carteira,
//! uma corretora ou um extrato de proventos numa página: conta (ou cria pelo nome), ativos
//! (sem duplicar pelo ticker), operações, posição numa data e proventos, **numa alteração
//! só**. Para a IA isso vira uma única proposta com o diff de todas as linhas.
//!
//! Tudo é "upsert": o que já existe é reaproveitado ou pulado, com o motivo no relatório.
//!
//! - **Ativos**: pelo ticker (sem diferenciar maiúsculas) ou, sem ticker, pelo nome.
//! - **Operações**: pulam se o `external_id` já está na conta ou se já existe a mesma
//!   operação (ativo, conta, data, lado, quantidade e preço). Vão em ordem de data, compras
//!   antes das vendas do mesmo dia.
//! - **Posição numa data** (quando a página só mostra quantidade e preço médio, como uma
//!   carteira consolidada): compara com a quantidade já registrada na conta até a data e
//!   lança só a diferença como compra ao preço médio informado, sem mexer no caixa. Se o
//!   registrado for maior, nada é lançado (vendas exigem a operação de verdade).
//! - **Proventos**: pulam se já existe o mesmo (ativo, conta, tipo, data de pagamento e
//!   valor bruto).

use std::collections::HashMap;

use rusqlite::{OptionalExtension, Transaction, params};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use ts_rs::TS;

use super::{IncomeInput, IncomeKind, IncomeStatus, TradeInput, TradeSide};
use crate::accounts::{AccountInput, AccountKind};
use crate::assets::AssetInput;
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};
use crate::ids::Id;

/// Itens por chamada (o agente divide carteiras e históricos longos).
pub const MAX_ITEMS: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BulkTrade {
    /// Ticker ou id do ativo (cadastrado ou em `assets` desta chamada).
    pub asset: String,
    pub date: String,
    pub side: TradeSide,
    pub quantity: String,
    pub price: String,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub fees: Option<i64>,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub taxes: Option<i64>,
    #[serde(default)]
    #[ts(optional)]
    pub note_number: Option<String>,
    /// Identificador da operação na corretora (evita duplicar).
    #[serde(default)]
    #[ts(optional)]
    pub external_id: Option<String>,
    /// Lançar no caixa da conta (padrão: sim).
    #[serde(default)]
    #[ts(optional)]
    pub cash_movement: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BulkPositionItem {
    pub asset: String,
    pub quantity: String,
    /// Preço médio (na moeda do ativo).
    pub average_price: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BulkPositions {
    /// Data da posição (`YYYY-MM-DD`).
    pub date: String,
    pub items: Vec<BulkPositionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BulkIncome {
    pub asset: String,
    pub kind: IncomeKind,
    pub pay_date: String,
    #[serde(default)]
    #[ts(optional)]
    pub ex_date: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub quantity: Option<String>,
    /// Centavos.
    #[ts(type = "number")]
    pub gross: i64,
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub tax_withheld: Option<i64>,
    #[serde(default)]
    #[ts(optional)]
    pub status: Option<IncomeStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BulkArgs {
    /// De onde vieram os dados (ex.: "carteira XP, página Minha carteira, 15/10/2026").
    pub source: String,
    /// Conta da corretora já cadastrada…
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
    /// …ou a conta pelo nome (usa a que existe com esse nome ou cria, com a instituição).
    #[serde(default)]
    #[ts(optional)]
    pub account: Option<AccountInput>,
    #[serde(default)]
    #[ts(optional)]
    pub assets: Option<Vec<AssetInput>>,
    #[serde(default)]
    #[ts(optional)]
    pub trades: Option<Vec<BulkTrade>>,
    #[serde(default)]
    #[ts(optional)]
    pub positions: Option<BulkPositions>,
    #[serde(default)]
    #[ts(optional)]
    pub income: Option<Vec<BulkIncome>>,
}

/// O que aconteceu com cada item.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BulkReport {
    pub account: Option<String>,
    pub assets_created: Vec<String>,
    pub assets_existing: Vec<String>,
    #[ts(type = "number")]
    pub trades_created: u32,
    #[ts(type = "number")]
    pub positions_adjusted: u32,
    #[ts(type = "number")]
    pub income_created: u32,
    /// `"<seção> <n>: <motivo>"`.
    pub skipped: Vec<String>,
    /// A conta foi criada pelo lote.
    #[serde(skip)]
    #[ts(skip)]
    pub account_created: bool,
}

impl BulkReport {
    pub fn total_created(&self) -> u32 {
        self.assets_created.len() as u32
            + self.trades_created
            + self.positions_adjusted
            + self.income_created
            + u32::from(self.account_created)
    }

    pub fn reason(&self, source: &str) -> String {
        let count = |key: &str, n: i64| t_args(key, &i18n::args(&[("n", n.into())]));
        let mut parts = vec![];
        if !self.assets_created.is_empty() {
            parts.push(count(
                "nucleo-carteira-lote-ativos",
                self.assets_created.len() as i64,
            ));
        }
        if self.trades_created > 0 {
            parts.push(count(
                "nucleo-carteira-lote-operacoes",
                self.trades_created.into(),
            ));
        }
        if self.positions_adjusted > 0 {
            parts.push(count(
                "nucleo-carteira-lote-posicoes",
                self.positions_adjusted.into(),
            ));
        }
        if self.income_created > 0 {
            parts.push(count(
                "nucleo-carteira-lote-proventos",
                self.income_created.into(),
            ));
        }
        let what = if parts.is_empty() {
            t("nucleo-carteira-lote-conta")
        } else {
            parts.join(", ")
        };
        t_args(
            "nucleo-carteira-motivo-lote",
            &i18n::args(&[
                ("itens", what.into()),
                ("fonte", source.chars().take(120).collect::<String>().into()),
            ]),
        )
    }
}

use super::records::dec;

/// "<seção> <n>: …" do relatório (`key` com `$n` e os outros argumentos).
fn skip(key: &str, n: usize, more: &[(&'static str, String)]) -> String {
    let mut a = i18n::args(&[("n", (n as i64).into())]);
    for (k, v) in more {
        a.insert(std::borrow::Cow::Borrowed(*k), v.clone().into());
    }
    t_args(key, &a)
}

fn validate(a: &BulkArgs) -> Result<()> {
    if a.source.trim().is_empty() {
        return Err(Error::invalid(t("nucleo-carteira-erro-lote-fonte")));
    }
    let n = a.assets.as_ref().map_or(0, Vec::len)
        + a.trades.as_ref().map_or(0, Vec::len)
        + a.positions.as_ref().map_or(0, |p| p.items.len())
        + a.income.as_ref().map_or(0, Vec::len);
    if n == 0 && a.account.is_none() {
        return Err(Error::invalid(t("nucleo-carteira-erro-lote-vazio")));
    }
    if n > MAX_ITEMS {
        return Err(Error::invalid(t_args(
            "nucleo-carteira-erro-lote-itens",
            &i18n::args(&[("maximo", (MAX_ITEMS as i64).into())]),
        )));
    }
    let needs_account = a.trades.as_ref().is_some_and(|t| !t.is_empty())
        || a.positions.as_ref().is_some_and(|p| !p.items.is_empty())
        || a.income.as_ref().is_some_and(|i| !i.is_empty());
    if needs_account && a.account_id.is_none() && a.account.is_none() {
        return Err(Error::invalid(t("nucleo-carteira-erro-lote-conta")));
    }
    if let Some(p) = &a.positions {
        crate::time::parse_date(&p.date)?;
    }
    Ok(())
}

/// Conta pelo nome (sem diferenciar maiúsculas, não arquivada) ou nova.
fn resolve_account(
    tx: &Transaction<'_>,
    a: &BulkArgs,
    today: chrono::NaiveDate,
    report: &mut BulkReport,
) -> Result<Option<Id>> {
    if let Some(id) = &a.account_id {
        crate::accounts::currency_and_kind(tx, id)?;
        report.account = Some(t_args(
            "nucleo-carteira-lote-conta-existente-id",
            &i18n::args(&[("id", id.clone().into())]),
        ));
        return Ok(Some(id.clone()));
    }
    let Some(input) = &a.account else {
        return Ok(None);
    };
    let existing: Option<Id> = tx
        .query_row(
            "SELECT id FROM account WHERE lower(trim(name)) = lower(trim(?1)) AND archived = 0
             ORDER BY created_at LIMIT 1",
            [&input.name],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(id) = existing {
        report.account = Some(t_args(
            "nucleo-carteira-lote-conta-existente",
            &i18n::args(&[("nome", input.name.trim().to_string().into())]),
        ));
        return Ok(Some(id));
    }
    let mut input = input.clone();
    if input.kind == AccountKind::CreditCard {
        return Err(Error::invalid(t("nucleo-carteira-erro-conta-cartao")));
    }
    input.opening_balance.get_or_insert(0);
    let id = crate::accounts::create(tx, &input, today)?;
    report.account = Some(t_args(
        "nucleo-carteira-lote-conta-nova",
        &i18n::args(&[("nome", input.name.trim().to_string().into())]),
    ));
    report.account_created = true;
    Ok(Some(id))
}

fn find_asset(tx: &Transaction<'_>, key: &str) -> Result<Option<Id>> {
    let key = key.trim();
    if key.is_empty() {
        return Ok(None);
    }
    let by_id: Option<Id> = tx
        .query_row("SELECT id FROM asset WHERE id = ?1", [key], |r| r.get(0))
        .optional()?;
    if by_id.is_some() {
        return Ok(by_id);
    }
    Ok(crate::assets::find_by_ticker(tx, key)?.map(|a| a.id))
}

fn cnpj_digits(s: &str) -> String {
    s.chars().filter(char::is_ascii_digit).collect()
}

/// Ativo sem ticker já cadastrado: pelo nome ou pelo nome oficial e, sem eles, pelo CNPJ na
/// mesma classe quando só um ativo o tem (o plano de previdência com apelido "Caixa VGBL" e
/// o nome do fundo na página).
fn same_asset(tx: &Transaction<'_>, a: &AssetInput) -> Result<Option<Id>> {
    let names = [Some(a.name.as_str()), a.official_name.as_deref()];
    for n in names.into_iter().flatten().filter(|n| !n.trim().is_empty()) {
        let found: Option<Id> = tx
            .query_row(
                "SELECT id FROM asset
                  WHERE lower(trim(name)) = lower(trim(?1))
                     OR lower(trim(official_name)) = lower(trim(?1))
                  ORDER BY archived LIMIT 1",
                [n],
                |r| r.get(0),
            )
            .optional()?;
        if found.is_some() {
            return Ok(found);
        }
    }
    let Some(cnpj) = a.cnpj.as_deref().map(cnpj_digits).filter(|c| c.len() == 14) else {
        return Ok(None);
    };
    let ids: Vec<Id> = {
        let mut stmt = tx.prepare(
            "SELECT id, cnpj FROM asset WHERE class = ?1 AND archived = 0 AND cnpj IS NOT NULL",
        )?;
        stmt.query_map([a.class.as_str()], |r| {
            Ok((r.get::<_, Id>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .filter(|(_, c)| cnpj_digits(c) == cnpj)
        .map(|(id, _)| id)
        .collect()
    };
    Ok(if ids.len() == 1 {
        ids.into_iter().next()
    } else {
        None
    })
}

fn resolve_assets(
    tx: &Transaction<'_>,
    list: &[AssetInput],
    report: &mut BulkReport,
) -> Result<HashMap<String, Id>> {
    let mut map = HashMap::new();
    for (i, a) in list.iter().enumerate() {
        let label = a.ticker.clone().unwrap_or_else(|| a.name.clone());
        let existing = match a.ticker.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
            Some(t) => crate::assets::find_by_ticker(tx, t)?.map(|x| x.id),
            None => same_asset(tx, a)?,
        };
        let id = match existing {
            Some(id) => {
                report.assets_existing.push(label.clone());
                id
            }
            None => match crate::assets::create(tx, a) {
                Ok(id) => {
                    report.assets_created.push(label.clone());
                    id
                }
                Err(e) => {
                    report.skipped.push(skip(
                        "nucleo-carteira-lote-ativo-erro",
                        i + 1,
                        &[("ativo", label.clone()), ("erro", e.message())],
                    ));
                    continue;
                }
            },
        };
        map.insert(label.trim().to_uppercase(), id.clone());
        map.insert(a.name.trim().to_uppercase(), id);
    }
    Ok(map)
}

fn asset_of(tx: &Transaction<'_>, map: &HashMap<String, Id>, key: &str) -> Result<Option<Id>> {
    if let Some(id) = map.get(&key.trim().to_uppercase()) {
        return Ok(Some(id.clone()));
    }
    find_asset(tx, key)
}

fn trade_exists(tx: &Transaction<'_>, t: &TradeInput) -> Result<bool> {
    if let Some(ext) = t.external_id.as_deref().filter(|e| !e.trim().is_empty()) {
        let hit: bool = tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM trade WHERE account_id = ?1 AND external_id = ?2)",
            params![t.account_id, ext],
            |r| r.get(0),
        )?;
        if hit {
            return Ok(true);
        }
    }
    let q = dec(&t.quantity, "nucleo-carteira-campo-quantidade")?
        .normalize()
        .to_string();
    let p = dec(&t.price, "nucleo-carteira-campo-preco")?
        .normalize()
        .to_string();
    Ok(tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM trade WHERE account_id = ?1 AND asset_id = ?2 AND date = ?3
           AND side = ?4 AND quantity = ?5 AND price = ?6)",
        params![
            t.account_id,
            t.asset_id,
            t.date,
            if t.side == TradeSide::Buy {
                "buy"
            } else {
                "sell"
            },
            q,
            p
        ],
        |r| r.get(0),
    )?)
}

/// Executa o lote dentro de uma transação (`ctx.write`).
pub fn apply(tx: &Transaction<'_>, a: &BulkArgs, today: chrono::NaiveDate) -> Result<BulkReport> {
    validate(a)?;
    let mut report = BulkReport::default();
    let account = resolve_account(tx, a, today, &mut report)?;
    let map = resolve_assets(tx, a.assets.as_deref().unwrap_or_default(), &mut report)?;

    // Operações: por data, compras antes das vendas do mesmo dia.
    let mut trades: Vec<(usize, &BulkTrade)> = a.trades.iter().flatten().enumerate().collect();
    trades.sort_by(|(_, x), (_, y)| {
        x.date
            .cmp(&y.date)
            .then((x.side == TradeSide::Sell).cmp(&(y.side == TradeSide::Sell)))
    });
    for (i, t) in trades {
        let n = i + 1;
        let Some(asset_id) = asset_of(tx, &map, &t.asset)? else {
            report.skipped.push(skip(
                "nucleo-carteira-lote-operacao-sem-ativo",
                n,
                &[("ativo", t.asset.clone())],
            ));
            continue;
        };
        let input = TradeInput {
            asset_id,
            account_id: account.clone().unwrap_or_default(),
            date: t.date.trim().to_string(),
            side: t.side,
            quantity: t.quantity.clone(),
            price: t.price.clone(),
            fees: t.fees,
            taxes: t.taxes,
            fx_rate: None,
            day_trade: None,
            note_number: t.note_number.clone(),
            notes: Some(t_args(
                "nucleo-carteira-nota-lote",
                &i18n::args(&[("fonte", a.source.trim().to_string().into())]),
            )),
            cash_movement: t.cash_movement,
            external_id: t.external_id.clone(),
        };
        match trade_exists(tx, &input) {
            Ok(true) => {
                report
                    .skipped
                    .push(skip("nucleo-carteira-lote-operacao-repetida", n, &[]));
                continue;
            }
            Ok(false) => {}
            Err(e) => {
                report.skipped.push(skip(
                    "nucleo-carteira-lote-operacao-erro",
                    n,
                    &[("erro", e.message())],
                ));
                continue;
            }
        }
        match super::records::create_trade(tx, &input) {
            Ok(_) => report.trades_created += 1,
            Err(e) => report.skipped.push(skip(
                "nucleo-carteira-lote-operacao-erro",
                n,
                &[("erro", e.message())],
            )),
        }
    }

    // Posição numa data: só a diferença, como compra ao preço médio, sem caixa.
    if let Some(p) = &a.positions {
        let account_id = account.clone().unwrap_or_default();
        for (i, item) in p.items.iter().enumerate() {
            let n = i + 1;
            let Some(asset_id) = asset_of(tx, &map, &item.asset)? else {
                report.skipped.push(skip(
                    "nucleo-carteira-lote-posicao-sem-ativo",
                    n,
                    &[("ativo", item.asset.clone())],
                ));
                continue;
            };
            let target = match dec(&item.quantity, "nucleo-carteira-campo-quantidade") {
                Ok(q) if q > Decimal::ZERO => q,
                Ok(_) => {
                    report
                        .skipped
                        .push(skip("nucleo-carteira-lote-posicao-quantidade", n, &[]));
                    continue;
                }
                Err(e) => {
                    report.skipped.push(skip(
                        "nucleo-carteira-lote-posicao-erro",
                        n,
                        &[("erro", e.message())],
                    ));
                    continue;
                }
            };
            let held = super::engine::quantity_held(tx, &asset_id, Some(&account_id), &p.date)?;
            if held == target {
                report.skipped.push(skip(
                    "nucleo-carteira-lote-posicao-confere",
                    n,
                    &[("ativo", item.asset.clone())],
                ));
                continue;
            }
            // Renda fixa por valor (caixinha, CDB de liquidez diária): saldo menor na tela é
            // resgate; vira a venda da diferença, sem mexer no caixa (o resgate já entrou na
            // conta). Ações e fundos com quantidade menor pedem a venda como operação.
            let fixed_income: bool = tx.query_row(
                "SELECT class = 'fixed_income' FROM asset WHERE id = ?1",
                [&asset_id],
                |r| r.get(0),
            )?;
            let side = if held > target {
                TradeSide::Sell
            } else {
                TradeSide::Buy
            };
            if held > target && !fixed_income {
                report.skipped.push(skip(
                    "nucleo-carteira-lote-posicao-maior",
                    n,
                    &[
                        ("ativo", item.asset.clone()),
                        ("registrado", held.normalize().to_string()),
                        ("informado", target.normalize().to_string()),
                    ],
                ));
                continue;
            }
            let diff = (target - held).abs();
            let input = TradeInput {
                asset_id,
                account_id: account_id.clone(),
                date: p.date.clone(),
                side,
                quantity: diff.normalize().to_string(),
                price: item.average_price.clone(),
                fees: None,
                taxes: None,
                fx_rate: None,
                day_trade: None,
                note_number: None,
                notes: Some(t_args(
                    "nucleo-carteira-nota-posicao",
                    &i18n::args(&[
                        ("data", p.date.clone().into()),
                        ("fonte", a.source.trim().to_string().into()),
                        ("diferenca", diff.normalize().to_string().into()),
                    ]),
                )),
                cash_movement: Some(false),
                external_id: Some(format!(
                    "posicao:{}:{}:{}",
                    p.date,
                    item.asset.trim(),
                    target.normalize()
                )),
            };
            if trade_exists(tx, &input)? {
                report
                    .skipped
                    .push(skip("nucleo-carteira-lote-posicao-repetida", n, &[]));
                continue;
            }
            match super::records::create_trade(tx, &input) {
                Ok(_) => report.positions_adjusted += 1,
                Err(e) => report.skipped.push(skip(
                    "nucleo-carteira-lote-posicao-erro",
                    n,
                    &[("erro", e.message())],
                )),
            }
        }
    }

    // Proventos.
    for (i, inc) in a.income.iter().flatten().enumerate() {
        let n = i + 1;
        let Some(asset_id) = asset_of(tx, &map, &inc.asset)? else {
            report.skipped.push(skip(
                "nucleo-carteira-lote-provento-sem-ativo",
                n,
                &[("ativo", inc.asset.clone())],
            ));
            continue;
        };
        let account_id = account.clone().unwrap_or_default();
        let kind = serde_json::to_value(inc.kind)?
            .as_str()
            .unwrap_or_default()
            .to_string();
        let dup: bool = tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM income_event WHERE asset_id = ?1 AND account_id = ?2
               AND kind = ?3 AND pay_date = ?4 AND gross = ?5)",
            params![asset_id, account_id, kind, inc.pay_date.trim(), inc.gross],
            |r| r.get(0),
        )?;
        if dup {
            report
                .skipped
                .push(skip("nucleo-carteira-lote-provento-repetido", n, &[]));
            continue;
        }
        let input = IncomeInput {
            asset_id,
            account_id,
            kind: inc.kind,
            ex_date: inc.ex_date.clone(),
            pay_date: inc.pay_date.trim().to_string(),
            quantity: inc.quantity.clone(),
            gross: inc.gross,
            tax_withheld: inc.tax_withheld,
            fx_rate: None,
            status: inc.status,
            cash_movement: None,
        };
        match super::records::create_income(tx, &input) {
            Ok(_) => report.income_created += 1,
            Err(e) => report.skipped.push(skip(
                "nucleo-carteira-lote-provento-erro",
                n,
                &[("erro", e.message())],
            )),
        }
    }
    Ok(report)
}

/// Relatório como JSON para a resposta (e para a proposta, quando a IA chama).
pub fn report_value(r: &BulkReport) -> Value {
    json!(r)
}
