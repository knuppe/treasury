//! Proventos digitados às pressas (`portfolio.quick_income`): a família informa só a data do
//! pagamento, o ticker e o valor recebido, e o resto sai do que o cofre já sabe.
//!
//! - **Ativo**: pelo ticker cadastrado.
//! - **Provento anunciado**: procura o pagamento na B3 (`research_dividend`, oficial) e na
//!   agenda do ativo (`dividend_event`: Yahoo, Fundamentus) até [`WINDOW_DAYS`] dias da data
//!   digitada; dele vêm o tipo, a data-com e o valor por ação.
//! - **Conta**: a corretora com posição na data-com (ou no pagamento). Com mais de uma, a
//!   que explica o valor (valor por ação × quantidade) ou a de maior posição.
//! - **Bruto e imposto**: o valor digitado é o recebido (líquido); com o provento achado, se
//!   ele bate com o bruto esperado (JCP sem o IR descontado), vale como bruto. Sem dado, o
//!   bruto sai do líquido pela retenção padrão do tipo (15% no JCP, 30% em dividendos
//!   americanos).
//! - **Status**: pagamento depois de hoje fica como anunciado (sem caixa).
//!
//! Cada linha volta com o que foi deduzido e por quê; a prévia (`quick_income_preview`) é só
//! leitura e a gravação registra as linhas prontas numa alteração só.

use std::collections::HashSet;

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::records::{credit_date, default_withholding, text};
use super::{IncomeInput, IncomeKind, IncomeStatus};
use crate::assets::{Asset, AssetClass};
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};
use crate::ids::Id;
use crate::time::{fmt_date, parse_date};

/// Linhas por chamada.
pub const MAX_ROWS: usize = 500;
/// Distância máxima, em dias corridos, entre a data digitada e o pagamento anunciado.
pub const WINDOW_DAYS: i64 = 5;
/// Diferença aceita entre o valor digitado e o esperado pelo provento anunciado.
const TOLERANCE: f64 = 0.02;

/// Uma linha digitada.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QuickIncomeRow {
    /// Data do pagamento (`YYYY-MM-DD`).
    pub date: String,
    pub ticker: String,
    /// Valor recebido em centavos, na moeda do ativo.
    #[ts(type = "number")]
    pub amount: i64,
    /// Força o tipo (senão vem do provento anunciado ou da classe do ativo).
    #[serde(default)]
    #[ts(optional)]
    pub kind: Option<IncomeKind>,
    /// Força a conta (senão a corretora com posição).
    #[serde(default)]
    #[ts(optional)]
    pub account_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QuickIncomeArgs {
    pub rows: Vec<QuickIncomeRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum QuickIncomeState {
    /// Pronta para registrar.
    Ready,
    /// Já existe o mesmo provento (ou repete uma linha anterior).
    Duplicate,
    /// Falta algo (ativo não cadastrado, sem conta…).
    Error,
}

/// O que o Treasury deduziu de uma linha.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QuickIncomeLine {
    /// Posição da linha na entrada (0 = primeira).
    pub row: u32,
    pub date: String,
    pub ticker: String,
    #[ts(type = "number")]
    pub amount: i64,
    pub state: QuickIncomeState,
    pub asset_id: Option<Id>,
    pub asset_name: Option<String>,
    pub currency: Option<String>,
    pub account_id: Option<Id>,
    pub account_name: Option<String>,
    pub kind: Option<IncomeKind>,
    pub ex_date: Option<String>,
    pub quantity: Option<String>,
    /// Valor por ação/cota do provento anunciado.
    pub per_share: Option<String>,
    #[ts(type = "number | null")]
    pub gross: Option<i64>,
    #[ts(type = "number | null")]
    pub tax_withheld: Option<i64>,
    #[ts(type = "number | null")]
    pub net: Option<i64>,
    pub status: Option<IncomeStatus>,
    /// De onde veio o provento anunciado (`b3_listados`, `yahoo`…), se achado.
    pub matched_source: Option<String>,
    /// O valor digitado confere com o anunciado × quantidade.
    pub verified: bool,
    /// Como cada campo foi deduzido, em frases curtas.
    pub notes: Vec<String>,
    pub error: Option<String>,
    /// A conta não pôde ser deduzida: escolher a conta resolve o erro da linha.
    pub needs_account: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QuickIncomeReport {
    #[ts(type = "number")]
    pub created: u32,
    pub lines: Vec<QuickIncomeLine>,
}

/// Provento anunciado candidato.
#[derive(Debug, Clone)]
struct Announced {
    kind: IncomeKind,
    ex_date: Option<String>,
    pay_date: NaiveDate,
    per_share: Decimal,
    source: String,
}

fn kind_of(s: &str) -> IncomeKind {
    match s {
        "dividend" => IncomeKind::Dividend,
        "jcp" => IncomeKind::Jcp,
        "fii_income" => IncomeKind::FiiIncome,
        "coupon" => IncomeKind::Coupon,
        "interest" => IncomeKind::Interest,
        "rent" => IncomeKind::Rent,
        "amortization" => IncomeKind::Amortization,
        _ => IncomeKind::Other,
    }
}

/// Tipo padrão pela classe (sem provento anunciado).
pub fn default_kind(class: AssetClass) -> IncomeKind {
    match class {
        AssetClass::Fii => IncomeKind::FiiIncome,
        AssetClass::FixedIncome | AssetClass::TreasuryBr => IncomeKind::Coupon,
        _ => IncomeKind::Dividend,
    }
}

/// Retenção padrão do tipo, como fração.
fn withholding_rate(kind: IncomeKind, class: AssetClass, on: NaiveDate) -> Decimal {
    const BASE: i64 = 1_000_000;
    Decimal::from(default_withholding(kind, class, BASE, on)) / Decimal::from(BASE)
}

/// Bruto e imposto a partir do líquido, pela retenção padrão na data `on`.
fn gross_from_net(net: i64, kind: IncomeKind, class: AssetClass, on: NaiveDate) -> (i64, i64) {
    let rate = withholding_rate(kind, class, on);
    if rate.is_zero() || rate >= Decimal::ONE {
        return (net, 0);
    }
    let gross = (Decimal::from(net) / (Decimal::ONE - rate))
        .round()
        .to_i64()
        .unwrap_or(net);
    (gross, gross - net)
}

/// Dinheiro no idioma da instalação (`R$ 12.345,67`, `USD 12.345,67`).
fn money(cents: i64, currency: &str) -> String {
    i18n::money(i18n::current(), cents, currency)
}

/// Data no idioma da instalação (`14/10/2026`, `Oct 14, 2026`).
fn br_date(d: &str) -> String {
    parse_date(d).map_or_else(|_| d.to_string(), |d| i18n::date(i18n::current(), d))
}

/// Decimal no idioma da instalação.
fn num(v: Decimal) -> String {
    super::records::fmt_dec(v)
}

fn announced(conn: &Connection, asset: &Asset, date: NaiveDate) -> Result<Vec<Announced>> {
    let from = fmt_date(date - chrono::Duration::days(WINDOW_DAYS));
    let to = fmt_date(date + chrono::Duration::days(WINDOW_DAYS));
    let mut out: Vec<Announced> = vec![];
    if let Some(t) = asset.ticker.as_deref() {
        let mut stmt = conn.prepare_cached(
            "SELECT kind, ex_date, value, per_shares, pay_date FROM research_dividend
             WHERE ticker = ?1 AND source = ?2 AND pay_date BETWEEN ?3 AND ?4
             ORDER BY pay_date, kind, seq",
        )?;
        let rows: Vec<(String, String, String, i64, String)> = stmt
            .query_map(
                params![
                    t.trim().to_uppercase(),
                    crate::research::b3_listados::SOURCE,
                    from,
                    to
                ],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )?
            .collect::<rusqlite::Result<_>>()?;
        for (kind, ex, v, per, pay) in rows {
            let (Ok(v), Ok(pay)) = (v.parse::<Decimal>(), parse_date(&pay)) else {
                continue;
            };
            out.push(Announced {
                kind: kind_of(&kind),
                ex_date: Some(ex),
                pay_date: pay,
                per_share: (v / Decimal::from(per.max(1))).normalize(),
                source: crate::research::b3_listados::SOURCE.into(),
            });
        }
    }
    let mut stmt = conn.prepare_cached(
        "SELECT kind, ex_date, amount_per_share, pay_date, source FROM dividend_event
         WHERE asset_id = ?1 AND estimated = 0 AND pay_date BETWEEN ?2 AND ?3 ORDER BY pay_date",
    )?;
    let rows: Vec<(String, Option<String>, String, String, String)> = stmt
        .query_map(params![asset.id, from, to], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    for (kind, ex, v, pay, source) in rows {
        let (Ok(v), Ok(pay)) = (v.parse::<Decimal>(), parse_date(&pay)) else {
            continue;
        };
        let kind = kind_of(&kind);
        // A agenda repete o que a B3 já trouxe: fica a oficial.
        let same = out.iter().any(|a| {
            a.kind == kind
                && a.pay_date == pay
                && (a.per_share - v).abs() <= a.per_share.abs() * Decimal::new(2, 2)
        });
        if !same {
            out.push(Announced {
                kind,
                ex_date: ex,
                pay_date: pay,
                per_share: v.normalize(),
                source,
            });
        }
    }
    Ok(out)
}

/// Contas com operações do ativo (as candidatas a ter recebido o provento).
fn accounts_with_trades(conn: &Connection, asset_id: &str) -> Result<Vec<Id>> {
    let mut stmt = conn.prepare_cached(
        "SELECT DISTINCT account_id FROM trade WHERE asset_id = ?1
         UNION SELECT DISTINCT t.account_id FROM trade t
           JOIN corporate_action c ON c.asset_id = t.asset_id
          WHERE c.new_asset_id = ?1 AND c.kind IN ('ticker_change', 'merger')",
    )?;
    let ids = stmt
        .query_map([asset_id], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(ids)
}

fn account_name(conn: &Connection, id: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT name FROM account WHERE id = ?1", [id], |r| r.get(0))
        .optional()?)
}

/// Valor esperado (bruto, em centavos) de um provento anunciado para uma quantidade.
fn expected_gross(per_share: Decimal, quantity: Decimal) -> i64 {
    (per_share * quantity * Decimal::from(100))
        .round()
        .to_i64()
        .unwrap_or(0)
}

fn rel_diff(a: i64, b: i64) -> f64 {
    if b == 0 {
        return f64::INFINITY;
    }
    (a - b).abs() as f64 / b.abs() as f64
}

/// Deduz uma linha (sem gravar). `seen` evita repetir linhas do mesmo lote.
fn resolve_row(
    conn: &Connection,
    n: usize,
    row: &QuickIncomeRow,
    today: NaiveDate,
    seen: &mut HashSet<(String, String, String, i64)>,
) -> Result<QuickIncomeLine> {
    let mut line = QuickIncomeLine {
        row: n as u32,
        date: row.date.trim().to_string(),
        ticker: row.ticker.trim().to_uppercase(),
        amount: row.amount,
        state: QuickIncomeState::Error,
        asset_id: None,
        asset_name: None,
        currency: None,
        account_id: None,
        account_name: None,
        kind: None,
        ex_date: None,
        quantity: None,
        per_share: None,
        gross: None,
        tax_withheld: None,
        net: None,
        status: None,
        matched_source: None,
        verified: false,
        notes: vec![],
        error: None,
        needs_account: false,
    };
    let fail = |mut l: QuickIncomeLine, msg: String| {
        l.error = Some(msg);
        Ok(l)
    };
    // Sem conta deduzível: a interface oferece a escolha da conta.
    let fail_account = |mut l: QuickIncomeLine, msg: String| {
        l.needs_account = true;
        fail(l, msg)
    };
    let Ok(date) = parse_date(&line.date) else {
        let msg = text("nucleo-base-erro-data", &[("valor", row.date.clone())]);
        return fail(line, msg);
    };
    if line.ticker.is_empty() {
        return fail(line, t("nucleo-carteira-rapido-erro-ticker"));
    }
    if row.amount <= 0 {
        return fail(line, t("nucleo-carteira-rapido-erro-valor"));
    }
    let Some(asset) = crate::assets::find_by_ticker(conn, &line.ticker)? else {
        let msg = text(
            "nucleo-carteira-rapido-erro-ativo",
            &[("ticker", line.ticker.clone())],
        );
        return fail(line, msg);
    };
    line.asset_id = Some(asset.id.clone());
    line.asset_name = Some(asset.name.clone());
    line.currency = Some(asset.currency.clone());

    // Contas candidatas e a quantidade de cada uma.
    let forced_account = row
        .account_id
        .as_deref()
        .map(str::trim)
        .filter(|a| !a.is_empty());
    let accounts: Vec<Id> = match forced_account {
        Some(a) => {
            if account_name(conn, a)?.is_none() {
                let msg = text(
                    "nucleo-carteira-rapido-erro-conta",
                    &[("conta", a.to_string())],
                );
                return fail(line, msg);
            }
            vec![a.to_string()]
        }
        None => accounts_with_trades(conn, &asset.id)?,
    };
    if accounts.is_empty() {
        let msg = text(
            "nucleo-carteira-rapido-erro-sem-operacoes",
            &[("ticker", line.ticker.clone())],
        );
        return fail_account(line, msg);
    }
    let held = |account: &str, on: &str| -> Result<Decimal> {
        super::engine::quantity_held(conn, &asset.id, Some(account), on)
    };

    // Provento anunciado × conta que melhor explicam o valor digitado.
    let mut candidates = announced(conn, &asset, date)?;
    if let Some(k) = row.kind {
        candidates.retain(|c| c.kind == k);
    }
    struct Pick {
        ann: Option<Announced>,
        account: Id,
        quantity: Decimal,
        /// Diferença relativa entre o digitado e o esperado.
        diff: f64,
        /// `diff` com um pouco de peso para a distância entre as datas.
        rank: f64,
        as_gross: bool,
    }
    let mut best: Option<Pick> = None;
    for c in &candidates {
        let on = c.ex_date.clone().unwrap_or_else(|| fmt_date(c.pay_date));
        for acc in &accounts {
            let q = held(acc, &on)?;
            if q <= Decimal::ZERO {
                continue;
            }
            let g = expected_gross(c.per_share, q);
            let credit = credit_date(
                c.ex_date.as_deref().and_then(|d| parse_date(d).ok()),
                c.pay_date,
            );
            let net = g - default_withholding(c.kind, asset.class, g, credit);
            let (d_net, d_gross) = (rel_diff(row.amount, net), rel_diff(row.amount, g));
            let as_gross = d_gross < d_net;
            let days = (c.pay_date - date).num_days().abs() as f64;
            let diff = d_net.min(d_gross);
            let rank = diff + days * 0.001;
            if best.as_ref().is_none_or(|b| rank < b.rank) {
                best = Some(Pick {
                    ann: Some(c.clone()),
                    account: acc.clone(),
                    quantity: q,
                    diff,
                    rank,
                    as_gross,
                });
            }
        }
    }
    if best.is_none() {
        // Sem provento anunciado (ou sem posição na data-com): a conta com mais posição no
        // pagamento; sem posição em nenhuma, a única conta com operações.
        let on = fmt_date(date);
        let mut by_qty = vec![];
        for acc in &accounts {
            by_qty.push((held(acc, &on)?, acc.clone()));
        }
        by_qty.sort_by_key(|b| std::cmp::Reverse(b.0));
        let with_position = by_qty.iter().filter(|(q, _)| *q > Decimal::ZERO).count();
        let (q, acc) = by_qty[0].clone();
        if q <= Decimal::ZERO && accounts.len() > 1 {
            let msg = text(
                "nucleo-carteira-rapido-erro-sem-posicao",
                &[("ticker", line.ticker.clone()), ("data", br_date(&on))],
            );
            return fail_account(line, msg);
        }
        if q <= Decimal::ZERO {
            line.notes.push(text(
                "nucleo-carteira-rapido-nota-sem-posicao",
                &[("ticker", line.ticker.clone())],
            ));
        } else if with_position > 1 {
            line.notes
                .push(t("nucleo-carteira-rapido-nota-maior-posicao"));
        }
        best = Some(Pick {
            ann: None,
            account: acc,
            quantity: q,
            diff: f64::INFINITY,
            rank: f64::INFINITY,
            as_gross: false,
        });
    }
    let pick = best.expect("escolha feita acima");

    let kind = row
        .kind
        .or(pick.ann.as_ref().map(|a| a.kind))
        .unwrap_or_else(|| default_kind(asset.class));
    // JCP: a alíquota vale pela data do crédito (data-com) ou do pagamento, a primeira.
    let on = match &pick.ann {
        Some(a) => credit_date(
            a.ex_date.as_deref().and_then(|d| parse_date(d).ok()),
            a.pay_date,
        ),
        None => date,
    };
    let (gross, tax) = match &pick.ann {
        Some(_) if pick.as_gross => {
            let tax = default_withholding(kind, asset.class, row.amount, on);
            (row.amount, tax)
        }
        _ => gross_from_net(row.amount, kind, asset.class, on),
    };
    line.kind = Some(kind);
    line.gross = Some(gross);
    line.tax_withheld = Some(tax);
    line.net = Some(gross - tax);
    line.account_name = account_name(conn, &pick.account)?;
    line.account_id = Some(pick.account.clone());
    if accounts.len() > 1 && pick.ann.is_some() && forced_account.is_none() {
        line.notes.push(text(
            "nucleo-carteira-rapido-nota-conta",
            &[("conta", line.account_name.clone().unwrap_or_default())],
        ));
    }
    match &pick.ann {
        Some(a) => {
            let label = if a.source == crate::research::b3_listados::SOURCE {
                "B3".to_string()
            } else {
                a.source.clone()
            };
            line.matched_source = Some(a.source.clone());
            line.ex_date = a.ex_date.clone();
            line.per_share = Some(a.per_share.to_string());
            line.quantity = Some(pick.quantity.normalize().to_string());
            line.verified = pick.diff <= TOLERANCE;
            let mut parts = vec![
                ("tipo", a.kind.label()),
                ("valor", num(a.per_share)),
                ("fonte", label),
                ("pagamento", br_date(&fmt_date(a.pay_date))),
            ];
            let key = match a.ex_date.as_deref() {
                Some(d) => {
                    parts.push(("datacom", br_date(d)));
                    "nucleo-carteira-rapido-nota-anunciado-com"
                }
                None => "nucleo-carteira-rapido-nota-anunciado",
            };
            line.notes.push(text(key, &parts));
            let expected = expected_gross(a.per_share, pick.quantity);
            if line.verified {
                line.notes.push(text(
                    "nucleo-carteira-rapido-nota-confere",
                    &[
                        ("quantidade", num(pick.quantity)),
                        ("valor", num(a.per_share)),
                        ("total", money(expected, &asset.currency)),
                    ],
                ));
            } else {
                line.notes.push(text(
                    "nucleo-carteira-rapido-nota-diferente",
                    &[
                        ("esperado", money(expected, &asset.currency)),
                        ("quantidade", num(pick.quantity)),
                    ],
                ));
            }
            if pick.as_gross && tax > 0 {
                line.notes.push(t("nucleo-carteira-rapido-nota-bruto"));
            }
        }
        None => {
            line.notes.push(text(
                "nucleo-carteira-rapido-nota-sem-anuncio",
                &[("data", br_date(&line.date)), ("tipo", kind.label())],
            ));
        }
    }
    if tax > 0 && !pick.as_gross {
        line.notes.push(text(
            "nucleo-carteira-rapido-nota-ir",
            &[
                ("imposto", money(tax, &asset.currency)),
                ("bruto", money(gross, &asset.currency)),
            ],
        ));
    }
    let status = if date > today {
        IncomeStatus::Announced
    } else {
        IncomeStatus::Received
    };
    if status == IncomeStatus::Announced {
        line.notes.push(t("nucleo-carteira-rapido-nota-futuro"));
    }
    line.status = Some(status);

    // Duplicados: no cofre e no próprio lote.
    let dup: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM income_event WHERE asset_id = ?1 AND account_id = ?2
           AND pay_date = ?3 AND (net = ?4 OR gross = ?5))",
        params![asset.id, pick.account, line.date, gross - tax, gross],
        |r| r.get(0),
    )?;
    let key = (
        asset.id.clone(),
        pick.account.clone(),
        line.date.clone(),
        gross,
    );
    if dup {
        line.state = QuickIncomeState::Duplicate;
        line.error = Some(t("nucleo-carteira-rapido-ja-registrado"));
    } else if !seen.insert(key) {
        line.state = QuickIncomeState::Duplicate;
        line.error = Some(t("nucleo-carteira-rapido-repetido"));
    } else {
        line.state = QuickIncomeState::Ready;
    }
    Ok(line)
}

fn validate(a: &QuickIncomeArgs) -> Result<()> {
    if a.rows.is_empty() {
        return Err(Error::invalid(t("nucleo-carteira-rapido-erro-sem-linhas")));
    }
    if a.rows.len() > MAX_ROWS {
        return Err(Error::invalid(text(
            "nucleo-carteira-rapido-erro-linhas",
            &[
                ("maximo", MAX_ROWS.to_string()),
                ("recebidas", a.rows.len().to_string()),
            ],
        )));
    }
    Ok(())
}

/// Prévia: o que cada linha vira, sem gravar nada.
pub fn preview(
    conn: &Connection,
    a: &QuickIncomeArgs,
    today: NaiveDate,
) -> Result<Vec<QuickIncomeLine>> {
    validate(a)?;
    let mut seen = HashSet::new();
    a.rows
        .iter()
        .enumerate()
        .map(|(n, r)| resolve_row(conn, n, r, today, &mut seen))
        .collect()
}

/// Registra as linhas prontas; as outras voltam com o motivo.
pub fn apply(
    tx: &Transaction<'_>,
    a: &QuickIncomeArgs,
    today: NaiveDate,
) -> Result<QuickIncomeReport> {
    let lines = preview(tx, a, today)?;
    let mut created = 0;
    for l in &lines {
        if l.state != QuickIncomeState::Ready {
            continue;
        }
        let (Some(asset_id), Some(account_id), Some(kind), Some(gross)) =
            (l.asset_id.clone(), l.account_id.clone(), l.kind, l.gross)
        else {
            continue;
        };
        super::records::create_income(
            tx,
            &IncomeInput {
                asset_id,
                account_id,
                kind,
                ex_date: l.ex_date.clone(),
                pay_date: l.date.clone(),
                quantity: l.quantity.clone(),
                gross,
                tax_withheld: l.tax_withheld,
                fx_rate: None,
                status: l.status,
                cash_movement: None,
            },
        )?;
        created += 1;
    }
    Ok(QuickIncomeReport { created, lines })
}

/// Motivo da alteração (histórico e proposta do agente).
pub fn reason(r: &QuickIncomeReport) -> String {
    let tickers: Vec<&str> = r
        .lines
        .iter()
        .filter(|l| l.state == QuickIncomeState::Ready)
        .map(|l| l.ticker.as_str())
        .collect();
    let mut uniq: Vec<&str> = vec![];
    for t in tickers {
        if !uniq.contains(&t) {
            uniq.push(t);
        }
    }
    let list = if uniq.len() > 6 {
        text(
            "nucleo-carteira-rapido-e-mais",
            &[
                ("lista", uniq[..6].join(", ")),
                ("n", (uniq.len() - 6).to_string()),
            ],
        )
    } else {
        uniq.join(", ")
    };
    t_args(
        "nucleo-carteira-rapido-motivo",
        &i18n::args(&[("n", i64::from(r.created).into()), ("lista", list.into())]),
    )
}
