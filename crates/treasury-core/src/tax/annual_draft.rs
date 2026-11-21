//! Prévia da declaração de ajuste anual a partir do que o cofre já sabe: holerites do ano
//! (sem 13º e PLR, que são de tributação exclusiva), despesas de saúde e de instrução
//! dedutíveis pelas categorias, aportes em PGBL e fundo de pensão, dependentes das fontes de
//! renda e o IRRF "dedo-duro" que sobrou na renda variável. A família confere e ajusta os
//! valores na tela; o cálculo é o de [`super::annual`].
//!
//! O que é dedutível (Lei 9.250/1995 art. 8º, II, a e b; Perguntas e Respostas IRPF 2026,
//! capítulos de despesas médicas e de instrução):
//! - saúde: plano de saúde, consultas (inclusive pediatra), exames, dentista e terapias e
//!   psicólogo; farmácia, vacinas, óculos e itens de saúde não são;
//! - instrução: escola, creche e faculdade; material, uniforme, transporte, cantina, idiomas
//!   e cursos livres não são.
//! - aposentadoria e pensão a partir do mês em que o titular faz 65 anos: R$ 1.903,98 por mês
//!   ficam isentos (Lei 7.713/1988 art. 6º, XV; pergunta 052).

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::annual::{self, Employment, Input, Output};
use crate::error::Result;
use crate::i18n::{args, t, t_args};

/// Subcategorias dedutíveis, pelos nomes em pt-BR (valem com as categorias em qualquer idioma).
const MEDICAL: [&str; 6] = [
    "Plano de saúde",
    "Consultas",
    "Pediatra",
    "Exames",
    "Dentista",
    "Terapias e psicólogo",
];
const EDUCATION: [&str; 3] = ["Escola", "Creche", "Faculdade"];

/// Parcela isenta mensal de aposentadoria e pensão a partir dos 65 anos (2025 e 2026).
const EXEMPT_65_MONTHLY: i64 = 190_398;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnnualDraft {
    pub input: Input,
    pub result: Output,
    /// Imposto mínimo das altas rendas (a partir de 2026), quando os rendimentos passam de
    /// R$ 600 mil. Estimativa sem o redutor do art. 16-B.
    pub irpfm: Option<annual::MinimumTax>,
    /// De onde veio cada valor e o que conferir.
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnnualDraftArgs {
    #[ts(type = "number")]
    pub year: i32,
    #[ts(optional)]
    pub member_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnnualComputeArgs {
    pub input: Input,
}

fn money(cents: i64) -> String {
    crate::i18n::money(crate::i18n::current(), cents, "BRL")
}

/// Categorias de despesa com estes nomes sob o grupo em pt-BR.
fn category_ids(conn: &Connection, parent: &str, names: &[&str]) -> Result<Vec<String>> {
    let mut out = vec![];
    for n in names {
        if let Some(id) = crate::categories::find_br_path(
            conn,
            Some(crate::categories::CategoryKind::Expense),
            &format!("{parent}{}{n}", crate::categories::PATH_SEP),
        )? {
            out.push(id);
        }
    }
    Ok(out)
}

/// Gasto do ano nas categorias, por membro do lançamento (ou da divisão).
fn spent_by_member(
    conn: &Connection,
    categories: &[String],
    year: i32,
    member_id: Option<&str>,
) -> Result<Vec<(Option<String>, i64)>> {
    if categories.is_empty() {
        return Ok(vec![]);
    }
    let list = categories
        .iter()
        .map(|c| format!("'{}'", c.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT member, -SUM(amount) FROM (
            SELECT COALESCE(s.member_id, t.member_id) AS member,
                   COALESCE(s.category_id, t.category_id) AS category,
                   COALESCE(s.amount, t.amount) AS amount
              FROM txn t LEFT JOIN txn_split s ON s.txn_id = t.id
             WHERE t.kind = 'expense' AND t.status IN ('cleared', 'reconciled')
               AND t.currency = 'BRL' AND substr(t.date, 1, 4) = ?1)
          WHERE category IN ({list}) AND (?2 IS NULL OR member = ?2)
          GROUP BY member"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params![year.to_string(), member_id], |r| {
            Ok((r.get::<_, Option<String>>(0)?, r.get::<_, i64>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows.into_iter().filter(|(_, v)| *v > 0).collect())
}

/// Aportes do ano em planos PGBL e fundos de pensão (dedutíveis até 12%).
fn private_pension(conn: &Connection, year: i32, member_id: Option<&str>) -> Result<i64> {
    let accounts = super::engine::member_accounts(conn, member_id)?;
    let mut stmt = conn.prepare(
        "SELECT t.account_id, t.quantity, t.price, t.fees FROM trade t
           JOIN pension_plan p ON p.asset_id = t.asset_id
          WHERE p.plan_type IN ('pgbl', 'efpc') AND t.side = 'buy' AND substr(t.date, 1, 4) = ?1",
    )?;
    let rows = stmt
        .query_map([year.to_string()], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut total = 0;
    for (acc, qty, price, fees) in rows {
        if accounts.as_ref().is_some_and(|a| !a.contains(&acc)) {
            continue;
        }
        let q: Decimal = qty.parse().unwrap_or_default();
        let p: Decimal = price.parse().unwrap_or_default();
        total += (q * p * Decimal::ONE_HUNDRED).round().to_i64().unwrap_or(0) + fees;
    }
    Ok(total)
}

/// Monta a prévia da declaração de `year` (ano-calendário) para o membro (ou a família).
pub fn draft(conn: &Connection, year: i32, member_id: Option<&str>) -> Result<AnnualDraft> {
    let mut notes = vec![];
    let mut input = Input {
        year,
        ..Default::default()
    };
    // Holerites do ano, por fonte, sem 13º, PLR e adiantamento (já incluído no mensal).
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.kind, s.dependents, m.birth_date, p.reference_month, p.gross,
                p.social_security, p.income_tax
           FROM payslip p JOIN income_source s ON s.id = p.source_id
           JOIN member m ON m.id = s.member_id
          WHERE substr(p.reference_month, 1, 4) = ?1 AND p.kind IN ('monthly', 'vacation', 'other')
            AND (?2 IS NULL OR s.member_id = ?2)
          ORDER BY s.name, p.reference_month",
    )?;
    let rows = stmt
        .query_map(params![year.to_string(), member_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, i64>(6)?,
                r.get::<_, i64>(7)?,
                r.get::<_, i64>(8)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut by_source: Vec<(
        String,
        String,
        Employment,
        std::collections::BTreeSet<String>,
    )> = vec![];
    let mut exempt_65 = 0i64;
    for (id, name, kind, dependents, birth, month, gross, ss, irrf) in rows {
        input.dependents = input.dependents.max(dependents.max(0) as u32);
        let mut taxable = gross;
        if matches!(kind.as_str(), "retirement" | "pension") {
            let turned_65 = birth
                .as_deref()
                .and_then(|b| NaiveDate::parse_from_str(b, "%Y-%m-%d").ok())
                .and_then(|b| b.with_year(b.year() + 65));
            let month_start = NaiveDate::parse_from_str(&format!("{month}-01"), "%Y-%m-%d").ok();
            let month_end = month_start.map(|d| crate::time::add_months(d, 1));
            if let (Some(t65), Some(end)) = (turned_65, month_end)
                && t65 < end
            {
                let cut = gross.min(EXEMPT_65_MONTHLY);
                taxable -= cut;
                exempt_65 += cut;
            }
        }
        match by_source.iter_mut().find(|s| s.0 == id) {
            Some(s) => {
                s.2.income += taxable;
                s.2.social_security += ss;
                s.2.withheld += irrf;
                s.3.insert(month);
            }
            None => by_source.push((
                id,
                name,
                Employment {
                    income: taxable,
                    social_security: ss,
                    withheld: irrf,
                },
                std::collections::BTreeSet::from([month]),
            )),
        }
    }
    if by_source.is_empty() {
        notes.push(t("nucleo-impostos-anual-nota-sem-holerite"));
    }
    for (_, name, e, months) in by_source {
        notes.push(t_args(
            "nucleo-impostos-anual-nota-holerites",
            &args(&[
                ("fonte", name.into()),
                ("meses", months.len().to_string().into()),
                ("valor", money(e.income).into()),
            ]),
        ));
        input.employment.push(e);
    }
    if exempt_65 > 0 {
        notes.push(t_args(
            "nucleo-impostos-anual-nota-65",
            &args(&[("valor", money(exempt_65).into())]),
        ));
    }
    // Saúde e instrução pelas categorias.
    let medical = category_ids(conn, "Saúde", &MEDICAL)?;
    input.medical = spent_by_member(conn, &medical, year, member_id)?
        .iter()
        .map(|(_, v)| v)
        .sum();
    let education = category_ids(conn, "Educação", &EDUCATION)?;
    input.education = spent_by_member(conn, &education, year, member_id)?
        .into_iter()
        .map(|(_, v)| v)
        .collect();
    if input.medical > 0 || !input.education.is_empty() {
        notes.push(t_args(
            "nucleo-impostos-anual-nota-despesas",
            &args(&[
                ("saude", money(input.medical).into()),
                (
                    "instrucao",
                    money(input.education.iter().sum::<i64>()).into(),
                ),
            ]),
        ));
    }
    input.private_pension = private_pension(conn, year, member_id)?;
    if input.private_pension > 0 {
        notes.push(t_args(
            "nucleo-impostos-anual-nota-pgbl",
            &args(&[("valor", money(input.private_pension).into())]),
        ));
    }
    // IRRF "dedo-duro" que sobrou na renda variável.
    let y = super::engine::year(conn, member_id, year)?;
    input.irrf_lei_11033 = y.irrf_lei_11033_left;
    if y.irrf_lei_11033_left > 0 {
        notes.push(t_args(
            "nucleo-impostos-anual-nota-dedo-duro",
            &args(&[("valor", money(y.irrf_lei_11033_left).into())]),
        ));
    }
    if member_id.is_none() {
        let members: i64 = conn
            .query_row("SELECT count(*) FROM member", [], |r| r.get(0))
            .optional()?
            .unwrap_or(0);
        if members > 1 {
            notes.push(t("nucleo-impostos-anual-nota-familia"));
        }
    }
    notes.push(t("nucleo-impostos-anual-nota-confira"));
    let result = annual::compute(&input);
    let irpfm = if year >= 2026 {
        let m = minimum_tax_input(conn, year, member_id, &result, exempt_65, &y)?;
        let r = annual::minimum_tax(&m);
        if r.applies {
            notes.push(t("nucleo-impostos-anual-nota-irpfm"));
        }
        Some(r)
    } else {
        None
    };
    Ok(AnnualDraft {
        input,
        result,
        irpfm,
        notes,
    })
}

/// Base e créditos do IRPFM a partir do cofre: rendimentos tributáveis, 13º e PLR, parcela
/// isenta dos 65 anos, dividendos e JCP de empresas brasileiras, ganho líquido em bolsa e o
/// resumo do exterior. Ficam fora (§1º): poupança, LCI/LCA/CRI/CRA, FII e Fiagro, heranças e
/// doações, indenizações e o ganho de capital fora de bolsa.
fn minimum_tax_input(
    conn: &Connection,
    year: i32,
    member_id: Option<&str>,
    result: &Output,
    exempt_65: i64,
    y: &super::TaxYear,
) -> Result<annual::MinimumTaxInput> {
    let (exclusive_income, exclusive_tax): (i64, i64) = conn.query_row(
        "SELECT coalesce(sum(p.gross), 0), coalesce(sum(p.income_tax), 0)
           FROM payslip p JOIN income_source s ON s.id = p.source_id
          WHERE substr(p.reference_month, 1, 4) = ?1
            AND p.kind IN ('thirteenth_1', 'thirteenth_2', 'plr')
            AND (?2 IS NULL OR s.member_id = ?2)",
        params![year.to_string(), member_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let accounts = super::engine::member_accounts(conn, member_id)?;
    let mut stmt = conn.prepare(
        "SELECT i.account_id, i.kind, i.gross, i.tax_withheld FROM income_event i
           JOIN asset a ON a.id = i.asset_id
          WHERE i.status = 'received' AND i.currency = 'BRL' AND i.kind IN ('dividend', 'jcp')
            AND a.class IN ('stock_br', 'bdr', 'etf_br') AND substr(i.pay_date, 1, 4) = ?1",
    )?;
    let rows = stmt
        .query_map([year.to_string()], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let (mut dividends, mut jcp, mut jcp_tax) = (0i64, 0i64, 0i64);
    for (acc, kind, gross, tax) in rows {
        if accounts.as_ref().is_some_and(|a| !a.contains(&acc)) {
            continue;
        }
        if kind == "jcp" {
            jcp += gross;
            jcp_tax += tax;
        } else {
            dividends += gross;
        }
    }
    let stock_gains: i64 = y
        .months
        .iter()
        .flat_map(|m| &m.categories)
        .map(|c| c.result)
        .sum::<i64>()
        .max(0);
    let foreign = (y.foreign.realized - y.foreign.loss_carried_in).max(0) + y.foreign.income;
    let dividend_withheld: i64 = y
        .dividends_over_limit
        .iter()
        .map(|d| d.withheld.max(d.expected_withholding))
        .sum();
    let best_tax = if result.best == "completo" {
        result.full.tax_due
    } else {
        result.simplified.tax_due
    };
    Ok(annual::MinimumTaxInput {
        year,
        total_income: result.taxable_income
            + exempt_65
            + exclusive_income
            + dividends
            + jcp
            + stock_gains
            + foreign,
        annual_tax: best_tax,
        exclusive_tax: exclusive_tax + jcp_tax,
        definitive_tax: y.total_tax + y.foreign.estimated_tax,
        dividend_withheld,
    })
}
