use serde::Deserialize;
use serde_json::Value;

use super::{EstimateArgs, IncomeReportInput, TaxCategoryPatch, TaxSettingsPatch};
use crate::api::{Ctx, parse_args, to_value};
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};

pub const READ_ONLY: &[&str] = &[
    "datasets",
    "dataset",
    "settings",
    "categories",
    "income_reports",
    "estimate",
    "city",
    "cities",
];

#[derive(Deserialize)]
struct CountryArg {
    country: String,
}

#[derive(Deserialize)]
struct UfArg {
    uf: String,
}

#[derive(Deserialize)]
struct CityArg {
    #[serde(default)]
    city: Option<super::city::City>,
}

#[derive(Deserialize)]
struct IdArg {
    id: String,
}

#[derive(Deserialize, Default)]
struct YearArg {
    #[serde(default)]
    year: Option<i32>,
}

pub fn handle(ctx: &mut Ctx<'_>, action: &str, args: Value) -> Result<Value> {
    match action {
        "datasets" => to_value(super::dataset::all_for(crate::country::is_brazil(
            ctx.conn(),
        ))?),
        "dataset" => {
            let d = super::dataset::get(&parse_args::<CountryArg>(args)?.country)?;
            if d.country == "BR" && !crate::country::is_brazil(ctx.conn()) {
                return Err(Error::invalid(t("nucleo-impostos-erro-brasil")));
            }
            to_value(d)
        }
        "settings" => to_value(super::settings(ctx.conn())?),
        // Preferência técnica (país, estado, dependentes): fora do histórico, só o usuário.
        "set_settings" => {
            if *ctx.actor != crate::api::Actor::User {
                return Err(Error::Forbidden(t(
                    "nucleo-impostos-erro-ajustes-interface",
                )));
            }
            let p: TaxSettingsPatch = parse_args(args)?;
            to_value(super::set_settings(ctx.conn(), &p)?)
        }
        // A cidade da família e a arrecadação dela (Brasil). A leitura devolve o que está
        // guardado; a escolha e a coleta no Tesouro são da pessoa, fora do histórico.
        "city" => to_value(super::city::state(ctx.conn())?),
        "cities" => to_value(super::city::cities(
            ctx.conn(),
            &parse_args::<UfArg>(args)?.uf,
        )?),
        "set_city" | "refresh_city" => {
            if *ctx.actor != crate::api::Actor::User {
                return Err(Error::Forbidden(t(
                    "nucleo-impostos-erro-ajustes-interface",
                )));
            }
            if action == "set_city" {
                let a: CityArg = parse_args(args)?;
                to_value(super::city::set_city(ctx.conn(), a.city)?)
            } else {
                to_value(super::city::refresh(ctx.conn())?)
            }
        }
        "categories" => to_value(super::categories(ctx.conn())?),
        "set_category" => {
            let p: TaxCategoryPatch = parse_args(args)?;
            ctx.write(&t("nucleo-impostos-motivo-grupo"), |tx| {
                super::set_category(tx, &p)
            })?;
            to_value(super::categories(ctx.conn())?)
        }
        "income_reports" => to_value(super::income_reports(
            ctx.conn(),
            parse_args::<YearArg>(args)?.year,
        )?),
        "save_income_report" => {
            let i: IncomeReportInput = parse_args(args)?;
            let reason = t_args(
                if i.id.is_some() {
                    "nucleo-impostos-motivo-informe-alterado"
                } else {
                    "nucleo-impostos-motivo-informe-lancado"
                },
                &i18n::args(&[("fonte", i.payer.trim().to_string().into())]),
            );
            ctx.write(&reason, |tx| super::save_income_report(tx, &i))?;
            to_value(super::income_reports(ctx.conn(), Some(i.year))?)
        }
        "delete_income_report" => {
            let a: IdArg = parse_args(args)?;
            ctx.write(&t("nucleo-impostos-motivo-informe-removido"), |tx| {
                super::delete_income_report(tx, &a.id)
            })?;
            Ok(Value::Null)
        }
        "estimate" => to_value(super::estimate(
            ctx.conn(),
            &parse_args::<EstimateArgs>(args)?,
        )?),
        _ => Err(Error::UnknownCommand(format!("tax_burden.{action}"))),
    }
}
