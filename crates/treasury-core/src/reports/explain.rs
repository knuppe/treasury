//! "Medido, nunca inventado": de onde vem cada número. Para cada [`Metric`] devolve o valor,
//! a fórmula no idioma da instalação e as fontes (saldo inicial, lançamentos, contas, taxas de câmbio).
//! A soma dos `amount` das fontes é igual a `value` sempre que a lista não foi truncada; as
//! fontes de câmbio têm `amount` 0 e só informam a taxa usada. Usado pelo MCP (`explain_value`).

use rusqlite::{Connection, OptionalExtension};
use serde::Deserialize;

use super::engine::{
    FlowQuery, Fx, Key, Params, basis_label, date_label, money, month_dates, month_label,
    normalize_month, percent, ratio, resolve_basis, today, validate_range,
};
use super::{ExplainArgs, Explanation, Metric, SourceKind, ValueSource};
use crate::budget::Basis;
use crate::categories::CategoryKind;
use crate::error::{Error, Result};
use crate::household::base_currency;
use crate::i18n::{Args, args, t, t_args};
use crate::ids::Id;
use crate::time::{fmt_date, parse_date};

#[derive(Deserialize)]
struct AccountParams {
    account_id: Id,
    #[serde(default)]
    as_of: Option<String>,
}

#[derive(Deserialize, Default)]
struct AsOfParams {
    #[serde(default)]
    as_of: Option<String>,
}

#[derive(Deserialize)]
struct FlowParams {
    from: String,
    to: String,
    #[serde(default)]
    basis: Option<Basis>,
    #[serde(default)]
    category_id: Option<Id>,
    #[serde(default)]
    member_id: Option<Id>,
}

#[derive(Deserialize)]
struct MonthParams {
    month: String,
    #[serde(default)]
    basis: Option<Basis>,
}

/// Padrão e teto de lançamentos listados.
const DEFAULT_LIMIT: u32 = 200;
const MAX_LIMIT: u32 = 10_000;

pub fn explain(conn: &Connection, a: &ExplainArgs) -> Result<Explanation> {
    let limit = a.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as i64;
    let params = a.params.clone();
    match a.metric {
        Metric::AccountBalance => account_balance(conn, crate::api::parse_args(params)?, limit),
        Metric::NetWorth => net_worth(conn, crate::api::parse_args(params)?),
        Metric::Spending => flow(
            conn,
            Metric::Spending,
            CategoryKind::Expense,
            crate::api::parse_args(params)?,
            limit,
        ),
        Metric::Income => flow(
            conn,
            Metric::Income,
            CategoryKind::Income,
            crate::api::parse_args(params)?,
            limit,
        ),
        Metric::SavingsRate => savings_rate(conn, crate::api::parse_args(params)?, limit),
    }
}

/// Taxa no idioma (`5.4321` → `5,4321` em pt-BR).
fn rate_label(rate: &str) -> String {
    crate::ai::alerts::decimal(crate::i18n::current(), rate)
}

/// Um texto do catálogo com argumentos de texto.
fn tx(key: &str, list: &[(&'static str, String)]) -> String {
    let a: Args = list
        .iter()
        .map(|(k, v)| (std::borrow::Cow::Borrowed(*k), v.clone().into()))
        .collect();
    t_args(key, &a)
}

/// "N lançamentos efetivados" (com o tipo, quando há).
fn count_label(key: &str, n: i64) -> String {
    t_args(key, &args(&[("n", n.into())]))
}

pub(super) fn txn_label(description: &str, payee: Option<&str>) -> String {
    let d = description.trim();
    if !d.is_empty() {
        d.to_string()
    } else {
        payee
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| t("nucleo-relatorios-sem-descricao"))
    }
}

/// Fontes das taxas de câmbio usadas (valor 0: só informam a taxa).
fn fx_sources(fx: &Fx<'_>) -> Vec<ValueSource> {
    fx.used
        .iter()
        .map(|u| ValueSource {
            kind: SourceKind::FxRate,
            id: None,
            label: tx(
                if u.estimated {
                    "nucleo-relatorios-fonte-cambio-estimado"
                } else {
                    "nucleo-relatorios-fonte-cambio"
                },
                &[
                    ("moeda", u.currency.clone()),
                    ("taxa", rate_label(&u.rate)),
                    ("alvo", fx.target.clone()),
                ],
            ),
            date: (!u.rate_date.is_empty()).then(|| u.rate_date.clone()),
            amount: 0,
            currency: fx.target.clone(),
            source: (!u.source.is_empty()).then(|| u.source.clone()),
            estimated: u.estimated,
        })
        .collect()
}

fn missing_note(missing: &[String]) -> String {
    if missing.is_empty() {
        String::new()
    } else {
        tx(
            "nucleo-relatorios-formula-sem-cambio",
            &[("moedas", missing.join(", "))],
        )
    }
}

// ---------------------------------------------------------------------------

fn account_balance(conn: &Connection, p: AccountParams, limit: i64) -> Result<Explanation> {
    let as_of = p.as_of.unwrap_or_else(|| fmt_date(today()));
    parse_date(&as_of)?;
    let (name, currency, opening, opening_date): (String, String, i64, String) = conn
        .query_row(
            "SELECT name, currency, opening_balance, opening_date FROM account WHERE id = ?1",
            [&p.account_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?
        .ok_or_else(|| {
            Error::not_found(tx(
                "nucleo-relatorios-conta",
                &[("id", p.account_id.clone())],
            ))
        })?;
    let opening_counts = opening_date <= as_of;
    let (sum, count): (i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(amount), 0), COUNT(*) FROM txn WHERE account_id = ?1 AND date <= ?2
           AND status IN ('cleared', 'reconciled')",
        rusqlite::params![p.account_id, as_of],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let value = sum + if opening_counts { opening } else { 0 };
    let mut sources = vec![];
    if opening_counts {
        sources.push(ValueSource {
            kind: SourceKind::Opening,
            id: Some(p.account_id.clone()),
            label: tx(
                "nucleo-relatorios-fonte-saldo-inicial",
                &[("conta", name.clone())],
            ),
            date: Some(opening_date.clone()),
            amount: opening,
            currency: currency.clone(),
            source: None,
            estimated: false,
        });
    }
    let mut stmt = conn.prepare_cached(
        "SELECT id, date, amount, description, payee FROM txn WHERE account_id = ?1 AND date <= ?2
           AND status IN ('cleared', 'reconciled') ORDER BY date DESC, id DESC LIMIT ?3",
    )?;
    let mut rows = stmt.query(rusqlite::params![p.account_id, as_of, limit])?;
    while let Some(r) = rows.next()? {
        let desc: String = r.get(3)?;
        let payee: Option<String> = r.get(4)?;
        sources.push(ValueSource {
            kind: SourceKind::Txn,
            id: Some(r.get(0)?),
            label: txn_label(&desc, payee.as_deref()),
            date: Some(r.get(1)?),
            amount: r.get(2)?,
            currency: currency.clone(),
            source: None,
            estimated: false,
        });
    }
    let txns = count_label("nucleo-relatorios-n-efetivados", count);
    let formula = if opening_counts {
        tx(
            "nucleo-relatorios-formula-saldo",
            &[
                ("inicial", money(opening, &currency)),
                ("lancamentos", txns),
                ("data", date_label(&as_of)),
                ("soma", money(sum, &currency)),
                ("valor", money(value, &currency)),
            ],
        )
    } else {
        tx(
            "nucleo-relatorios-formula-saldo-abertura",
            &[
                ("lancamentos", txns),
                ("data", date_label(&as_of)),
                ("valor", money(value, &currency)),
                ("abertura", date_label(&opening_date)),
                ("inicial", money(opening, &currency)),
            ],
        )
    };
    Ok(Explanation {
        metric: Metric::AccountBalance,
        value,
        currency,
        formula,
        sources,
        truncated: count > limit,
        estimated: false,
        ratio: None,
        count,
        missing_fx: vec![],
    })
}

fn net_worth(conn: &Connection, p: AsOfParams) -> Result<Explanation> {
    let as_of = p.as_of.unwrap_or_else(|| fmt_date(today()));
    let nw = super::net_worth(conn, &as_of)?;
    let base = nw.base_currency.clone();
    let mut fx = Fx::new(conn, &base);
    let mut sources = vec![];
    for a in &nw.accounts {
        let mut label = a.name.clone();
        let mut source = None;
        if a.currency != base {
            let original = money(a.balance, &a.currency);
            label = match fx.convert(a.balance, &a.currency, &as_of)? {
                Some(c) => {
                    source = c.source;
                    tx(
                        "nucleo-relatorios-fonte-conta-convertida",
                        &[
                            ("conta", label),
                            ("valor", original),
                            ("taxa", rate_label(&c.rate)),
                            ("convertido", money(c.amount, &base)),
                        ],
                    )
                }
                None => tx(
                    "nucleo-relatorios-fonte-conta-sem-cambio",
                    &[("conta", label), ("valor", original)],
                ),
            };
        }
        sources.push(ValueSource {
            kind: SourceKind::Account,
            id: Some(a.account_id.clone()),
            label,
            date: Some(as_of.clone()),
            amount: a.balance_base.unwrap_or(0),
            currency: base.clone(),
            source,
            estimated: a.fx_estimated,
        });
    }
    if nw.investments != 0 {
        sources.push(ValueSource {
            kind: SourceKind::Portfolio,
            id: None,
            label: t("nucleo-relatorios-fonte-carteira"),
            date: Some(as_of.clone()),
            amount: nw.investments,
            currency: base.clone(),
            source: None,
            estimated: false,
        });
    }
    if nw.physical != 0 {
        let loaded = crate::physical::load_all(conn).unwrap_or_default();
        for item in crate::physical::net_worth_items(&loaded, parse_date(&as_of)?) {
            let Some(c) = fx.convert(item.value, &item.currency, &as_of)? else {
                continue;
            };
            let mut label = format!("{}: {}", item.name, item.formula);
            if item.currency != base {
                label = tx(
                    "nucleo-relatorios-fonte-convertida",
                    &[
                        ("rotulo", label),
                        ("taxa", rate_label(&c.rate)),
                        ("convertido", money(c.amount, &base)),
                    ],
                );
            }
            sources.push(ValueSource {
                kind: SourceKind::Physical,
                id: Some(item.id),
                label,
                date: Some(as_of.clone()),
                amount: c.amount,
                currency: base.clone(),
                source: Some(t(if item.estimated {
                    "nucleo-relatorios-fonte-estimativa"
                } else {
                    "nucleo-relatorios-fonte-avaliacao"
                })),
                estimated: item.estimated,
            });
        }
    }
    if nw.debts != 0 {
        sources.push(ValueSource {
            kind: SourceKind::Loan,
            id: None,
            label: t("nucleo-relatorios-fonte-dividas"),
            date: Some(as_of.clone()),
            amount: -nw.debts,
            currency: base.clone(),
            source: None,
            estimated: false,
        });
    }
    sources.extend(fx_sources(&fx));
    let n = nw.accounts.len() as i64;
    let mut a = args(&[("n", n.into())]);
    for (k, v) in [
        ("ativos", money(nw.assets, &base)),
        ("passivos", money(nw.liabilities, &base)),
        ("valor", money(nw.net, &base)),
        ("data", date_label(&as_of)),
        ("sem_cambio", missing_note(&nw.missing_fx)),
    ] {
        a.insert(k.into(), v.into());
    }
    let formula = t_args("nucleo-relatorios-formula-patrimonio", &a);
    Ok(Explanation {
        metric: Metric::NetWorth,
        value: nw.net,
        currency: base,
        formula,
        sources,
        truncated: false,
        estimated: nw.estimated,
        ratio: None,
        count: n,
        missing_fx: nw.missing_fx,
    })
}

/// Lança uma fonte por lançamento (somando as partes da divisão que entram no filtro).
/// `negate`: a fonte vale −valor (despesa como gasto positivo).
fn txn_sources(
    conn: &Connection,
    q: &FlowQuery<'_>,
    fx: &mut Fx<'_>,
    limit: i64,
    negate: bool,
) -> Result<(Vec<ValueSource>, bool)> {
    let mut p = Params::default();
    let eff = q.eff_expr();
    let fw = q.sql_tail(&mut p, false);
    let lim = p.bind(limit + 1);
    let sql = format!(
        "SELECT t.id, t.date, {eff} AS eff, t.description, t.payee, t.currency,
                SUM(COALESCE(s.amount, t.amount)), COUNT(s.id)
         {fw} GROUP BY t.id ORDER BY eff DESC, t.id DESC LIMIT {lim}"
    );
    let mut stmt = conn.prepare_cached(&sql)?;
    let mut rows = stmt.query(rusqlite::params_from_iter(p.0.iter()))?;
    let mut out = vec![];
    let mut truncated = false;
    while let Some(r) = rows.next()? {
        if out.len() as i64 >= limit {
            truncated = true;
            break;
        }
        let id: String = r.get(0)?;
        let date: String = r.get(1)?;
        let eff: String = r.get(2)?;
        let desc: String = r.get(3)?;
        let payee: Option<String> = r.get(4)?;
        let currency: String = r.get(5)?;
        let raw: i64 = r.get(6)?;
        let parts: i64 = r.get(7)?;
        let signed = if negate { -raw } else { raw };
        let mut label = txn_label(&desc, payee.as_deref());
        if parts > 0 {
            let mut a = args(&[("n", parts.into())]);
            a.insert("rotulo".into(), label.into());
            label = t_args("nucleo-relatorios-fonte-partes", &a);
        }
        if eff != date {
            label = tx(
                "nucleo-relatorios-fonte-fatura",
                &[
                    ("rotulo", label),
                    ("compra", date_label(&date)),
                    ("vencimento", date_label(&eff)),
                ],
            );
        }
        let (amount, source, estimated) = match fx.convert(signed, &currency, &date)? {
            Some(c) => {
                if currency != fx.target {
                    label = tx(
                        "nucleo-relatorios-fonte-conta-convertida",
                        &[
                            ("conta", label),
                            ("valor", money(signed, &currency)),
                            ("taxa", rate_label(&c.rate)),
                            ("convertido", money(c.amount, &fx.target)),
                        ],
                    );
                }
                (c.amount, c.source, c.estimated)
            }
            None => {
                label = tx(
                    "nucleo-relatorios-fonte-lancamento-sem-cambio",
                    &[("rotulo", label), ("valor", money(signed, &currency))],
                );
                (0, None, false)
            }
        };
        out.push(ValueSource {
            kind: SourceKind::Txn,
            id: Some(id),
            label,
            date: Some(eff),
            amount,
            currency: fx.target.clone(),
            source: if currency != fx.target { source } else { None },
            estimated,
        });
    }
    Ok((out, truncated))
}

fn flow(
    conn: &Connection,
    metric: Metric,
    kind: CategoryKind,
    p: FlowParams,
    limit: i64,
) -> Result<Explanation> {
    validate_range(&p.from, &p.to)?;
    let basis = resolve_basis(conn, p.basis)?;
    let base = base_currency(conn)?;
    let cats = match &p.category_id {
        Some(c) => Some(crate::categories::with_descendants(
            conn,
            std::slice::from_ref(c),
        )?),
        None => None,
    };
    let members = p.member_id.clone().map(|m| vec![m]);
    let mut q = FlowQuery::new(&p.from, &p.to, basis);
    q.kind = Some(kind);
    q.category_ids = cats.as_deref();
    q.member_ids = members.as_deref();
    let mut fx = Fx::new(conn, &base);
    let total = q.aggregate(conn, &mut fx, Key::None, Key::None)?.total();
    let value = total.amount(kind);
    let count = total.count(kind);
    let (mut sources, truncated) =
        txn_sources(conn, &q, &mut fx, limit, kind == CategoryKind::Expense)?;
    sources.extend(fx_sources(&fx));
    let missing: Vec<String> = total.missing_fx.iter().cloned().collect();

    let mut filters = String::new();
    if let Some(c) = &p.category_id {
        let path = super::engine::CategoryTree::load(conn)?
            .get(c)
            .map(|c| c.path.clone());
        filters.push(' ');
        filters.push_str(&tx(
            "nucleo-relatorios-filtro-categoria",
            &[("categoria", path.unwrap_or_else(|| c.clone()))],
        ));
    }
    if let Some(m) = &p.member_id {
        let name: Option<String> = conn
            .query_row("SELECT name FROM member WHERE id = ?1", [m], |r| r.get(0))
            .optional()?;
        filters.push(' ');
        filters.push_str(&tx(
            "nucleo-relatorios-filtro-membro",
            &[("membro", name.unwrap_or_else(|| m.clone()))],
        ));
    }
    let (what, rule) = match kind {
        CategoryKind::Expense => (
            count_label("nucleo-relatorios-n-despesas", count),
            t("nucleo-relatorios-regra-despesa"),
        ),
        CategoryKind::Income => (
            count_label("nucleo-relatorios-n-receitas", count),
            t("nucleo-relatorios-regra-receita"),
        ),
    };
    let converted = if fx.used.is_empty() {
        String::new()
    } else {
        tx(
            "nucleo-relatorios-formula-convertidos",
            &[("moeda", base.clone())],
        )
    };
    let formula = tx(
        "nucleo-relatorios-formula-fluxo",
        &[
            ("lancamentos", what),
            ("de", date_label(&p.from)),
            ("ate", date_label(&p.to)),
            ("base", basis_label(basis)),
            ("filtros", filters),
            ("regra", rule),
            ("valor", money(value, &base)),
            ("convertidos", converted),
            ("sem_cambio", missing_note(&missing)),
        ],
    );
    Ok(Explanation {
        metric,
        value,
        currency: base,
        formula,
        sources,
        truncated,
        estimated: fx.estimated,
        ratio: None,
        count,
        missing_fx: missing,
    })
}

fn savings_rate(conn: &Connection, p: MonthParams, limit: i64) -> Result<Explanation> {
    let month = normalize_month(&p.month)?;
    let (from, to) = month_dates(&month)?;
    let basis = resolve_basis(conn, p.basis)?;
    let base = base_currency(conn)?;
    let q = FlowQuery::new(&from, &to, basis);
    let mut fx = Fx::new(conn, &base);
    let total = q.aggregate(conn, &mut fx, Key::None, Key::None)?.total();
    let value = total.income - total.expense;
    let rate = ratio(value, total.income);
    let count = total.income_count + total.expense_count;
    // Receita − despesa = soma dos valores com sinal: cada fonte vale o próprio lançamento.
    let (mut sources, truncated) = txn_sources(conn, &q, &mut fx, limit, false)?;
    sources.extend(fx_sources(&fx));
    let missing: Vec<String> = total.missing_fx.iter().cloned().collect();
    let txns = count_label("nucleo-relatorios-n-efetivados", count);
    let formula = match rate {
        Some(r) => tx(
            "nucleo-relatorios-formula-poupanca",
            &[
                ("mes", month_label(&month)),
                ("base", basis_label(basis)),
                ("receitas", money(total.income, &base)),
                ("despesas", money(total.expense, &base)),
                ("taxa", percent(r)),
                ("valor", money(value, &base)),
                ("lancamentos", txns),
                ("sem_cambio", missing_note(&missing)),
            ],
        ),
        None => tx(
            "nucleo-relatorios-formula-poupanca-sem-receitas",
            &[
                ("mes", month_label(&month)),
                ("valor", money(value, &base)),
                ("lancamentos", txns),
                ("sem_cambio", missing_note(&missing)),
            ],
        ),
    };
    Ok(Explanation {
        metric: Metric::SavingsRate,
        value,
        currency: base,
        formula,
        sources,
        truncated,
        estimated: fx.estimated,
        ratio: rate,
        count,
        missing_fx: missing,
    })
}
