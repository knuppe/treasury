use rusqlite::params;
use serde::Deserialize;
use serde_json::Value;

use super::*;
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n;

pub const READ_ONLY: &[&str] = &[
    "list",
    "get",
    "schedule",
    "summary",
    "simulate_prepayment",
    "simulate_portability",
    "solve_rate",
    "simulate_anticipation",
    "statement_preview",
    "payment_candidates",
    "payoff_plan",
    "market_rates",
];

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

#[derive(Deserialize, Default)]
struct ListArgs {
    #[serde(default)]
    include_archived: bool,
}

#[derive(Deserialize)]
struct UpdateArgs {
    id: String,
    name: Option<String>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    contract_number: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    notes: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    debit_account_id: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    category_id: Option<Option<String>>,
    insurance_monthly: Option<i64>,
    fees_monthly: Option<i64>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    property_value: Option<Option<i64>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    mip_rate: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    dfi_rate: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    mip_amount: Option<Option<i64>>,
    #[serde(default, deserialize_with = "crate::patch::double")]
    dfi_amount: Option<Option<i64>>,
    archived: Option<bool>,
    /// Só a taxa efetiva % a.a. (ex.: a implícita no extrato do contrato).
    annual_rate: Option<String>,
    /// Todos os termos do contrato (valor, taxa ou parcela, sistema, indexador, datas,
    /// prazo…): recalcula o cronograma mantendo parcelas pagas, amortizações e
    /// antecipações.
    terms: Option<super::LoanInput>,
}

#[derive(Deserialize)]
struct ExtraArgs {
    loan_id: String,
    date: String,
    amount: i64,
    mode: PrepayMode,
    /// Lançar a saída na conta (padrão: conta de débito do contrato, se houver).
    #[serde(default)]
    from_account_id: Option<String>,
    /// Não lançar a saída em conta nenhuma (ex.: amortização com FGTS).
    #[serde(default)]
    without_txn: bool,
}

#[derive(Deserialize)]
struct LinkArgs {
    installment_id: String,
    txn_id: String,
}

#[derive(Deserialize)]
struct InstallmentArg {
    installment_id: String,
}

#[derive(Deserialize)]
struct PaidArgs {
    installment_id: String,
    date: String,
    amount: Option<i64>,
    txn_id: Option<String>,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "list" => to_value(list(
            ctx.conn(),
            parse_args::<ListArgs>(args)?.include_archived,
        )?),
        "get" => to_value(view(ctx.conn(), &parse_args::<IdArg>(args)?.id)?),
        "schedule" => to_value(installments(ctx.conn(), &parse_args::<IdArg>(args)?.id)?),
        "payoff_plan" => to_value(super::payoff::payoff_plan(
            ctx.conn(),
            &parse_args::<super::payoff::PayoffArgs>(args)?,
            today_local(),
        )?),
        "solve_rate" => to_value(solve_rate(&parse_args::<RateArgs>(args)?)?),
        "statement_preview" => to_value(super::statement::preview(
            ctx.conn(),
            &parse_args::<super::statement::StatementArgs>(args)?,
        )?),
        "import_statement" => {
            let a: super::statement::StatementArgs = parse_args(args)?;
            // Como proposta, o resultado da simulação vai no relatório (o resumo usa).
            let mut seen = None;
            let r = ctx.write(
                &i18n::t("nucleo-planejamento-divida-motivo-extrato"),
                |tx| {
                    let v = super::statement::import(tx, &a)?;
                    seen = Some(to_value(&v)?);
                    Ok(v)
                },
            );
            match r {
                Ok(r) => to_value(r.value),
                Err(e @ Error::Proposed { .. }) => {
                    if let Some(v) = seen {
                        ctx.set_report(v);
                    }
                    Err(e)
                }
                Err(e) => Err(e),
            }
        }
        "summary" => {
            let conn = ctx.conn();
            let loans = list(conn, false)?;
            // Os totais saem na moeda base da família: contrato em outra moeda entra convertido
            // pelo câmbio mais recente (sem câmbio, pelo valor da moeda dele, como antes).
            let base = crate::household::base_currency(conn)?;
            let today = crate::time::fmt_date(crate::time::today_local());
            let in_base = |amount: i64, currency: &str| -> Result<i64> {
                if currency == base {
                    return Ok(amount);
                }
                Ok(crate::fx::convert(conn, amount, currency, &base, &today)?
                    .map_or(amount, |c| c.amount))
            };
            let mut total_balance = 0;
            let mut monthly_commitment = 0;
            let mut interest_remaining = 0;
            for l in &loans {
                total_balance += in_base(l.balance, &l.currency)?;
                if let Some(i) = &l.next_installment {
                    monthly_commitment += in_base(i.total, &l.currency)?;
                }
                interest_remaining += in_base(l.interest_remaining, &l.currency)?;
            }
            to_value(DebtSummary {
                total_balance,
                monthly_commitment,
                interest_remaining,
                loans,
            })
        }
        "create" => {
            let i: LoanInput = parse_args(args)?;
            let reason = i18n::t_args(
                "nucleo-planejamento-divida-motivo-cadastrado",
                &i18n::args(&[("nome", i.name.trim().to_string().into())]),
            );
            let r = ctx.write(&reason, |tx| create(tx, &i))?;
            to_value(view(ctx.conn(), &r.value)?)
        }
        "update" => {
            let a: UpdateArgs = parse_args(args)?;
            ctx.write(
                &i18n::t("nucleo-planejamento-divida-motivo-alterado"),
                |tx| {
                    if let Some(t) = &a.terms {
                        super::update_terms(tx, &a.id, t)?;
                    }
                    let now = now_utc();
                    if let Some(n) = &a.name {
                        tx.execute(
                            "UPDATE loan SET name = ?2, updated_at = ?3 WHERE id = ?1",
                            params![a.id, n.trim(), now],
                        )?;
                    }
                    if let Some(n) = &a.contract_number {
                        tx.execute(
                            "UPDATE loan SET contract_number = ?2, updated_at = ?3 WHERE id = ?1",
                            params![a.id, clean_contract_number(n.as_deref()), now],
                        )?;
                    }
                    if let Some(n) = &a.notes {
                        tx.execute(
                            "UPDATE loan SET notes = ?2, updated_at = ?3 WHERE id = ?1",
                            params![a.id, n, now],
                        )?;
                    }
                    if let Some(acc) = &a.debit_account_id {
                        tx.execute(
                            "UPDATE loan SET debit_account_id = ?2, updated_at = ?3 WHERE id = ?1",
                            params![a.id, acc, now],
                        )?;
                    }
                    if let Some(c) = &a.category_id {
                        super::set_category(tx, &a.id, c.as_deref())?;
                    }
                    if let Some(v) = a.archived {
                        tx.execute(
                            "UPDATE loan SET archived = ?2, updated_at = ?3 WHERE id = ?1",
                            params![a.id, v as i64, now],
                        )?;
                    }
                    let mut regen = false;
                    if let Some(r) = &a.annual_rate {
                        let d: rust_decimal::Decimal =
                            r.trim().replace(',', ".").parse().map_err(|_| {
                                Error::invalid(i18n::t_args(
                                    "nucleo-planejamento-divida-erro-taxa-invalida",
                                    &i18n::args(&[("taxa", r.clone().into())]),
                                ))
                            })?;
                        if d < rust_decimal::Decimal::ZERO || d > rust_decimal::Decimal::from(1000)
                        {
                            return Err(Error::invalid(i18n::t(
                                "nucleo-planejamento-divida-erro-taxa-intervalo",
                            )));
                        }
                        tx.execute(
                            "UPDATE loan SET annual_rate = ?2, updated_at = ?3 WHERE id = ?1",
                            params![a.id, d.normalize().to_string(), now],
                        )?;
                        regen = true;
                    }
                    if let Some(v) = a.insurance_monthly {
                        tx.execute(
                            "UPDATE loan SET insurance_monthly = ?2, updated_at = ?3 WHERE id = ?1",
                            params![a.id, v, now],
                        )?;
                        regen = true;
                    }
                    if let Some(v) = a.fees_monthly {
                        tx.execute(
                            "UPDATE loan SET fees_monthly = ?2, updated_at = ?3 WHERE id = ?1",
                            params![a.id, v, now],
                        )?;
                        regen = true;
                    }
                    if a.property_value.is_some()
                        || a.mip_rate.is_some()
                        || a.dfi_rate.is_some()
                        || a.mip_amount.is_some()
                        || a.dfi_amount.is_some()
                    {
                        let cur = super::housing_insurance(tx, &a.id)?;
                        let mip_rate = a.mip_rate.clone().unwrap_or(cur.mip_rate);
                        let dfi_rate = a.dfi_rate.clone().unwrap_or(cur.dfi_rate);
                        super::set_housing_insurance(
                            tx,
                            &a.id,
                            &super::HousingInput {
                                property_value: a.property_value.unwrap_or(cur.property),
                                mip_rate: mip_rate.as_deref(),
                                dfi_rate: dfi_rate.as_deref(),
                                mip_amount: a.mip_amount.unwrap_or(cur.mip_amount),
                                dfi_amount: a.dfi_amount.unwrap_or(cur.dfi_amount),
                            },
                        )?;
                        regen = true;
                    }
                    if regen {
                        regenerate(tx, &a.id)?;
                    }
                    Ok(())
                },
            )?;
            to_value(view(ctx.conn(), &a.id)?)
        }
        "delete" => {
            let a: IdArg = parse_args(args)?;
            ctx.write(
                &i18n::t("nucleo-planejamento-divida-motivo-removido"),
                |tx| {
                    if tx.execute("DELETE FROM loan WHERE id = ?1", [&a.id])? == 0 {
                        return Err(Error::not_found(i18n::t(
                            "nucleo-planejamento-divida-contrato",
                        )));
                    }
                    Ok(())
                },
            )?;
            Ok(Value::Null)
        }
        "add_extra_payment" => {
            let a: ExtraArgs = parse_args(args)?;
            if a.amount <= 0 {
                return Err(Error::invalid(i18n::t(
                    "nucleo-planejamento-divida-erro-amortizacao-positiva",
                )));
            }
            parse_date(&a.date)?;
            ctx.write(&i18n::t("nucleo-planejamento-divida-motivo-amortizacao"), |tx| {
                let acc: Option<String> = match a.from_account_id.clone() {
                    _ if a.without_txn => None,
                    Some(x) => Some(x),
                    None => tx.query_row("SELECT debit_account_id FROM loan WHERE id = ?1", [&a.loan_id], |r| r.get(0))?,
                };
                let txn_id = match acc {
                    Some(acc) => {
                        let (cur, _) = crate::accounts::currency_and_kind(tx, &acc)?;
                        let name: String = tx.query_row("SELECT name FROM loan WHERE id = ?1", [&a.loan_id], |r| r.get(0))?;
                        let n = crate::transactions::NewTxn::new(
                            &acc,
                            &a.date,
                            -a.amount,
                            &cur,
                            crate::transactions::TxnKind::Adjustment,
                            &i18n::t_args(
                                "nucleo-planejamento-divida-lancamento-amortizacao",
                                &i18n::args(&[("contrato", name.into())]),
                            ),
                        );
                        Some(crate::transactions::insert(tx, &n)?)
                    }
                    None => None,
                };
                let now = now_utc();
                tx.execute(
                    "INSERT INTO loan_extra_payment (id, loan_id, date, amount, mode, txn_id, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
                    params![new_id(), a.loan_id, a.date, a.amount, kind_str(&a.mode), txn_id, now],
                )?;
                regenerate(tx, &a.loan_id)
            })?;
            to_value(view(ctx.conn(), &a.loan_id)?)
        }
        "simulate_prepayment" => to_value(simulate_prepayment(
            ctx.conn(),
            &parse_args::<PrepayArgs>(args)?,
        )?),
        "simulate_anticipation" => to_value(simulate_anticipation(
            ctx.conn(),
            &parse_args::<AnticipateArgs>(args)?,
        )?),
        "anticipate" => {
            #[derive(Deserialize)]
            struct Args {
                #[serde(flatten)]
                a: AnticipateArgs,
                #[serde(default)]
                from_account_id: Option<String>,
                #[serde(default)]
                without_txn: bool,
            }
            let Args {
                a,
                from_account_id,
                without_txn,
            } = parse_args(args)?;
            let r = ctx.write(
                &i18n::t("nucleo-planejamento-divida-motivo-antecipadas"),
                |tx| {
                    let sim = simulate_anticipation(tx, &a)?;
                    let acc: Option<String> = match from_account_id.clone() {
                        _ if without_txn => None,
                        Some(x) => Some(x),
                        None => tx.query_row(
                            "SELECT debit_account_id FROM loan WHERE id = ?1",
                            [&a.loan_id],
                            |r| r.get(0),
                        )?,
                    };
                    let txn_id = match acc {
                        Some(acc) => {
                            let (cur, _) = crate::accounts::currency_and_kind(tx, &acc)?;
                            let name: String = tx.query_row(
                                "SELECT name FROM loan WHERE id = ?1",
                                [&a.loan_id],
                                |r| r.get(0),
                            )?;
                            let n = crate::transactions::NewTxn::new(
                                &acc,
                                &a.date,
                                -sim.to_pay,
                                &cur,
                                crate::transactions::TxnKind::Adjustment,
                                &i18n::t_args(
                                    "nucleo-planejamento-divida-lancamento-antecipacao",
                                    &i18n::args(&[
                                        ("n", sim.installments.len().into()),
                                        ("contrato", name.into()),
                                    ]),
                                ),
                            );
                            Some(crate::transactions::insert(tx, &n)?)
                        }
                        None => None,
                    };
                    apply_anticipation(tx, &a, txn_id.as_deref())
                },
            )?;
            to_value(r.value)
        }
        "mark_paid_until" => {
            #[derive(Deserialize)]
            struct Args {
                loan_id: String,
                before: String,
            }
            let a: Args = parse_args(args)?;
            parse_date(&a.before)?;
            let r = ctx.write(
                &i18n::t("nucleo-planejamento-divida-motivo-vencidas-pagas"),
                |tx| mark_paid_until(tx, &a.loan_id, &a.before),
            )?;
            to_value(serde_json::json!({ "marked": r.value }))
        }
        "simulate_portability" => to_value(simulate_portability(
            ctx.conn(),
            &parse_args::<PortabilityArgs>(args)?,
        )?),
        "reconcile" => {
            let a: IdArg = parse_args(args)?;
            let r = ctx.write(
                &i18n::t("nucleo-planejamento-divida-motivo-conciliadas"),
                |tx| reconcile(tx, &a.id),
            )?;
            to_value(serde_json::json!({ "matched": r.value }))
        }
        "record_payment" => {
            let a: super::PaymentArgs = parse_args(args)?;
            to_value(
                ctx.write(
                    &i18n::t("nucleo-planejamento-divida-motivo-pagamento"),
                    |tx| super::record_payment(tx, &a),
                )?
                .value,
            )
        }
        "mark_paid" => {
            let a: PaidArgs = parse_args(args)?;
            parse_date(&a.date)?;
            ctx.write(&i18n::t("nucleo-planejamento-divida-motivo-parcela-paga"), |tx| {
                let total: i64 = tx.query_row("SELECT total FROM loan_installment WHERE id = ?1", [&a.installment_id], |r| r.get(0))?;
                tx.execute(
                    "UPDATE loan_installment SET status = 'paid', paid_date = ?2, paid_amount = ?3, txn_id = ?4, updated_at = ?5 WHERE id = ?1",
                    params![a.installment_id, a.date, a.amount.unwrap_or(total), a.txn_id, now_utc()],
                )?;
                Ok(())
            })?;
            Ok(Value::Null)
        }
        "payment_candidates" => to_value(super::link::payment_candidates(
            ctx.conn(),
            &parse_args::<InstallmentArg>(args)?.installment_id,
        )?),
        "link_payment" => {
            let a: LinkArgs = parse_args(args)?;
            let (loan_id, name, number) = super::link::describe(ctx.conn(), &a.installment_id)?;
            let reason = i18n::t_args(
                "nucleo-planejamento-divida-motivo-parcela-ligada",
                &i18n::args(&[
                    ("numero", number.to_string().into()),
                    ("contrato", name.into()),
                ]),
            );
            let mut seen = None;
            let r = ctx.write(&reason, |tx| {
                super::link::link_payment(tx, &a.installment_id, &a.txn_id)?;
                seen = installments(tx, &loan_id)?
                    .into_iter()
                    .find(|i| i.id == a.installment_id)
                    .map(|i| to_value(&i))
                    .transpose()?;
                Ok(())
            });
            if let Err(e @ Error::Proposed { .. }) = r {
                if let Some(v) = seen {
                    ctx.set_report(v);
                }
                return Err(e);
            }
            r?;
            let inst = installments(ctx.conn(), &loan_id)?
                .into_iter()
                .find(|i| i.id == a.installment_id)
                .ok_or_else(|| Error::not_found(i18n::t("nucleo-planejamento-divida-parcela")))?;
            to_value(inst)
        }
        "unmark_paid" => {
            let a: IdArg = parse_args(args)?;
            ctx.write(&i18n::t("nucleo-planejamento-divida-motivo-pagamento-desfeito"), |tx| {
                tx.execute(
                    "UPDATE loan_installment SET status = 'planned', paid_date = NULL, paid_amount = NULL, txn_id = NULL, updated_at = ?2 WHERE id = ?1",
                    params![a.id, now_utc()],
                )?;
                Ok(())
            })?;
            Ok(Value::Null)
        }
        _ => Err(Error::UnknownCommand(format!("debt.{action}"))),
    }
}
