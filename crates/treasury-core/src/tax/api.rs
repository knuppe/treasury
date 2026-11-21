use rusqlite::{OptionalExtension, params};
use serde::Deserialize;
use serde_json::Value;

use super::*;
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};
use crate::ids::new_id;
use crate::time::now_utc;

pub const READ_ONLY: &[&str] = &[
    "month",
    "year",
    "list_darfs",
    "crypto_custody",
    "annual_draft",
    "annual_compute",
    "darf_charges",
    "capital_gain",
    "physical_gain",
    "plr",
    "carne_leao",
];

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

#[derive(Deserialize, Default)]
struct ListArgs {
    member_id: Option<String>,
    year: Option<i32>,
}

fn row_to_darf(r: &rusqlite::Row<'_>) -> rusqlite::Result<Darf> {
    let status: String = r.get("status")?;
    Ok(Darf {
        id: r.get("id")?,
        member_id: r.get("member_id")?,
        period: r.get("period")?,
        revenue_code: r.get("revenue_code")?,
        principal: r.get("principal")?,
        fine: r.get("fine")?,
        interest: r.get("interest")?,
        total: r.get("total")?,
        due_date: r.get("due_date")?,
        status: match status.as_str() {
            "paid" => DarfStatus::Paid,
            "cancelled" => DarfStatus::Cancelled,
            _ => DarfStatus::Pending,
        },
        paid_at: r.get("paid_at")?,
        txn_id: r.get("txn_id")?,
        notes: r.get("notes")?,
    })
}

fn get_darf(conn: &rusqlite::Connection, id: &str) -> Result<Darf> {
    conn.query_row("SELECT * FROM darf WHERE id = ?1", [id], row_to_darf)
        .optional()?
        .ok_or_else(|| Error::not_found(format!("DARF {id}")))
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "month" => {
            let a: TaxMonthArgs = parse_args(args)?;
            to_value(engine::month(ctx.conn(), a.member_id.as_deref(), &a.month)?)
        }
        "year" => {
            let a: TaxYearArgs = parse_args(args)?;
            to_value(engine::year(ctx.conn(), a.member_id.as_deref(), a.year)?)
        }
        "annual_draft" => {
            let a: super::annual_draft::AnnualDraftArgs = parse_args(args)?;
            if !(2000..=2100).contains(&a.year) {
                return Err(Error::invalid(t("nucleo-impostos-ir-erro-ano")));
            }
            to_value(super::annual_draft::draft(
                ctx.conn(),
                a.year,
                a.member_id.as_deref(),
            )?)
        }
        "annual_compute" => {
            let a: super::annual_draft::AnnualComputeArgs = parse_args(args)?;
            to_value(super::annual::compute(&a.input))
        }
        "darf_charges" => {
            let a: DarfChargesArgs = parse_args(args)?;
            let d = get_darf(ctx.conn(), &a.id)?;
            let paid = match &a.paid_at {
                Some(p) => crate::time::parse_date(p)?,
                None => crate::time::today_local(),
            };
            let due = crate::time::parse_date(&d.due_date)?;
            to_value(super::late::for_darf(ctx.conn(), d.principal, due, paid)?)
        }
        "capital_gain" => {
            let a: super::gcap::Input = parse_args(args)?;
            to_value(super::gcap::compute(&a)?)
        }
        "physical_gain" => {
            let a: super::gcap::PhysicalGainArgs = parse_args(args)?;
            to_value(super::gcap::for_physical(ctx.conn(), &a)?)
        }
        "plr" => {
            let a: super::exclusive::PlrArgs = parse_args(args)?;
            to_value(super::exclusive::plr(&a)?)
        }
        "carne_leao" => {
            let a: super::exclusive::CarneLeaoArgs = parse_args(args)?;
            to_value(super::exclusive::carne_leao(&a)?)
        }
        "crypto_custody" => to_value(engine::crypto_custody(ctx.conn())?),
        "set_crypto_custody" => {
            let a: SetCryptoCustodyArgs = parse_args(args)?;
            crate::accounts::currency_and_kind(ctx.conn(), &a.account_id)?;
            ctx.write(&t("nucleo-impostos-ir-motivo-custodia-cripto"), |tx| {
                engine::set_crypto_custody(tx, &a.account_id, a.abroad)
            })?;
            to_value(engine::crypto_custody(ctx.conn())?)
        }
        "list_darfs" => {
            let a: ListArgs = parse_args(args)?;
            let mut stmt = ctx.conn().prepare(
                "SELECT * FROM darf WHERE (?1 IS NULL OR member_id = ?1) AND (?2 IS NULL OR substr(period, 1, 4) = ?2)
                 ORDER BY period DESC",
            )?;
            let rows = stmt
                .query_map(
                    params![a.member_id, a.year.map(|y| y.to_string())],
                    row_to_darf,
                )?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            to_value(rows)
        }
        "generate_darf" => {
            // Escrita sensível: só pela interface nesta fase.
            if *ctx.actor != crate::api::Actor::User {
                return Err(Error::Forbidden(t("nucleo-impostos-ir-erro-gerar-darf")));
            }
            let a: TaxMonthArgs = parse_args(args)?;
            let m = engine::month(ctx.conn(), a.member_id.as_deref(), &a.month)?;
            let crypto_code = super::rules::TaxRules::for_year(2026).capital_gain_darf_code;
            // DARF de cripto (ganho de capital, 4600) ou o de renda variável (6015).
            let (amount, revenue_code, due_date, existing, below_minimum) =
                match a.revenue_code.as_deref() {
                    Some(code) if code == crypto_code => {
                        let c = m.crypto.clone().ok_or_else(|| {
                            Error::invalid(t("nucleo-impostos-ir-erro-sem-imposto"))
                        })?;
                        let below = c.darf_amount == 0 && c.carried_out > 0;
                        (c.darf_amount, c.revenue_code, c.due_date, c.darf_id, below)
                    }
                    Some(code) if code != m.revenue_code => {
                        return Err(Error::invalid(t("nucleo-impostos-ir-erro-sem-imposto")));
                    }
                    _ => (
                        m.darf_amount,
                        m.revenue_code.clone(),
                        m.due_date.clone(),
                        m.darf_id.clone(),
                        m.status == MonthStatus::BelowMinimum,
                    ),
                };
            if amount <= 0 {
                return Err(Error::invalid(t(if below_minimum {
                    "nucleo-impostos-ir-erro-abaixo-minimo"
                } else {
                    "nucleo-impostos-ir-erro-sem-imposto"
                })));
            }
            if existing.is_some() {
                return Err(Error::conflict(t("nucleo-impostos-ir-erro-darf-existe")));
            }
            let id = new_id();
            let now = now_utc();
            let reason = t_args(
                "nucleo-impostos-ir-motivo-darf",
                &i18n::args(&[
                    ("codigo", revenue_code.clone().into()),
                    ("mes", m.month.clone().into()),
                ]),
            );
            ctx.write(&reason, |tx| {
                tx.execute(
                    "INSERT INTO darf (id, member_id, period, revenue_code, principal, total, due_date, status, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, 'pending', ?7, ?7)",
                    params![id, a.member_id, m.month, revenue_code, amount, due_date, now],
                )?;
                Ok(())
            })?;
            to_value(get_darf(ctx.conn(), &id)?)
        }
        "mark_darf_paid" => {
            if *ctx.actor != crate::api::Actor::User {
                return Err(Error::Forbidden(t(
                    "nucleo-impostos-ir-erro-pagar-interface",
                )));
            }
            let a: MarkDarfPaidArgs = parse_args(args)?;
            let mut d = get_darf(ctx.conn(), &a.id)?;
            let paid_on = crate::time::parse_date(&a.paid_at)?;
            // Pago depois do vencimento: multa e juros (Lei 9.430/1996 art. 61) entram no total.
            let due = crate::time::parse_date(&d.due_date)?;
            let charges = super::late::for_darf(ctx.conn(), d.principal, due, paid_on)?;
            d.fine = charges.fine;
            d.interest = charges.interest;
            d.total = charges.total;
            let reason = t_args(
                "nucleo-impostos-ir-motivo-pagamento",
                &i18n::args(&[("mes", d.period.clone().into())]),
            );
            let description = t_args(
                "nucleo-impostos-ir-lancamento-darf",
                &i18n::args(&[
                    ("codigo", d.revenue_code.clone().into()),
                    ("mes", d.period.clone().into()),
                ]),
            );
            ctx.write(&reason, |tx| {
                let txn_id = match (&a.txn_id, &a.from_account_id) {
                    (Some(t), _) => Some(t.clone()),
                    (None, Some(acc)) => {
                        let (cur, _) = crate::accounts::currency_and_kind(tx, acc)?;
                        // Pelos nomes em pt-BR: vale com as categorias em qualquer idioma.
                        let cat = crate::categories::find_br_path(
                            tx,
                            None,
                            &format!(
                                "Impostos e taxas{}Imposto de renda",
                                crate::categories::PATH_SEP
                            ),
                        )?;
                        let mut n = crate::transactions::NewTxn::new(
                            acc,
                            &a.paid_at,
                            -d.total,
                            &cur,
                            crate::transactions::TxnKind::Expense,
                            &description,
                        );
                        n.category_id = cat;
                        Some(crate::transactions::insert(tx, &n)?)
                    }
                    _ => None,
                };
                tx.execute(
                    "UPDATE darf SET status = 'paid', paid_at = ?2, txn_id = ?3, updated_at = ?4,
                            fine = ?5, interest = ?6, total = ?7 WHERE id = ?1",
                    params![
                        a.id,
                        a.paid_at,
                        txn_id,
                        now_utc(),
                        d.fine,
                        d.interest,
                        d.total
                    ],
                )?;
                Ok(())
            })?;
            to_value(get_darf(ctx.conn(), &a.id)?)
        }
        "cancel_darf" => {
            let a: IdArg = parse_args(args)?;
            let d = get_darf(ctx.conn(), &a.id)?;
            if d.status == DarfStatus::Paid {
                return Err(Error::conflict(t("nucleo-impostos-ir-erro-darf-pago")));
            }
            let reason = t_args(
                "nucleo-impostos-ir-motivo-cancelado",
                &i18n::args(&[("mes", d.period.clone().into())]),
            );
            ctx.write(&reason, |tx| {
                tx.execute(
                    "UPDATE darf SET status = 'cancelled', updated_at = ?2 WHERE id = ?1",
                    params![a.id, now_utc()],
                )?;
                Ok(())
            })?;
            to_value(get_darf(ctx.conn(), &a.id)?)
        }
        _ => Err(Error::UnknownCommand(format!("tax.{action}"))),
    }
}
