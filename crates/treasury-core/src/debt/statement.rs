//! Extrato do contrato: o histórico que o banco mostra (data, prestação, amortização, juros,
//! seguros, taxas, saldo devedor, correção monetária e valor pago), colado da planilha ou do
//! internet banking. Serve para contratos que o cálculo puro não reproduz (amortizações
//! extraordinárias sem o valor, TR, mudanças de prazo):
//!
//! - cada linha vira a parcela daquele mês, **paga**, com os valores reais;
//! - as amortizações extraordinárias aparecem pelo salto do saldo entre um mês e o seguinte
//!   (saldo anterior + correção − amortização − saldo novo) e ficam na parcela do mês;
//! - o cronograma futuro é projetado a partir do último saldo real, com o prazo remanescente
//!   (informado, ou calculado pela prestação da última linha);
//! - a taxa implícita nos juros do extrato é comparada com a do contrato (os bancos costumam
//!   informar a taxa **nominal**, ao mês = anual ÷ 12).

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{AmortSystem, parse_kind, regenerate, schedule};
use crate::error::{Error, Result};
use crate::i18n::{self, args, t, t_args};
use crate::ids::{Id, new_id};
use crate::time::{fmt_date, now_utc};

/// Uma linha do extrato, em centavos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StatementRow {
    /// Data do vencimento/pagamento (`YYYY-MM-DD`).
    pub date: String,
    /// Prestação (encargo) do mês.
    #[ts(type = "number")]
    pub total: i64,
    #[ts(type = "number")]
    pub amortization: i64,
    #[ts(type = "number")]
    pub interest: i64,
    #[serde(default)]
    #[ts(type = "number")]
    pub insurance: i64,
    #[serde(default)]
    #[ts(type = "number")]
    pub fees: i64,
    /// Correção monetária do saldo no mês (TR, IPCA…).
    #[serde(default)]
    #[ts(type = "number")]
    pub adjustment: i64,
    /// Saldo devedor depois da parcela.
    #[ts(type = "number")]
    pub balance_after: i64,
    /// Valor efetivamente pago (sem ele, a prestação).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub paid: Option<i64>,
    /// Número da parcela no extrato do banco: recomeça em 1 a cada versão do contrato
    /// (depois de cada amortização extraordinária).
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StatementArgs {
    pub id: Id,
    /// Texto colado da planilha ou do internet banking (com o cabeçalho, de preferência).
    #[serde(default)]
    #[ts(optional)]
    pub text: Option<String>,
    /// Ou as linhas já separadas (agente).
    #[serde(default)]
    #[ts(optional)]
    pub rows: Option<Vec<StatementRow>>,
    /// Parcelas que faltam depois da última linha (sem ele, calculado pela prestação).
    #[serde(default)]
    #[ts(optional)]
    pub remaining_term: Option<u32>,
    /// Marcar como pagas as parcelas anteriores ao extrato que estão em aberto (padrão: sim).
    #[serde(default)]
    #[ts(optional)]
    pub mark_previous_paid: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InferredExtra {
    pub date: String,
    #[ts(type = "number")]
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StatementPreview {
    pub rows: Vec<StatementRow>,
    pub first_date: Option<String>,
    pub last_date: Option<String>,
    /// Saldo devedor depois da última linha.
    #[ts(type = "number | null")]
    pub last_balance: Option<i64>,
    /// Prazo remanescente usado na projeção (informado ou calculado).
    #[ts(type = "number | null")]
    pub remaining_term: Option<u32>,
    /// Amortizações extraordinárias achadas pelo salto do saldo.
    pub extras: Vec<InferredExtra>,
    /// Taxa implícita nos juros do extrato: % ao mês, nominal e efetiva ao ano.
    pub implied_monthly_pct: Option<String>,
    pub implied_nominal_annual_pct: Option<String>,
    pub implied_effective_annual_pct: Option<String>,
    /// Taxa do contrato (efetiva % a.a.).
    pub contract_rate_pct: String,
    /// Linhas que não foram entendidas (texto) e outros avisos.
    pub warnings: Vec<String>,
    /// Os juros do extrato não batem com a taxa do contrato (o aviso vai em `warnings`).
    #[serde(default)]
    pub rate_mismatch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StatementResult {
    pub preview: StatementPreview,
    #[ts(type = "number")]
    pub imported: u32,
    /// Parcelas anteriores ao extrato marcadas como pagas.
    #[ts(type = "number")]
    pub marked_previous: u32,
}

/// "R$ 3.835,41" → 383541; "-12,5" → -1250.
fn money(s: &str) -> Option<i64> {
    let t: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || matches!(c, ',' | '.' | '-'))
        .collect();
    if t.is_empty() || t == "-" {
        return None;
    }
    // Formato brasileiro (1.234,56) ou com ponto decimal (1234.56).
    let normalized = if t.contains(',') {
        t.replace('.', "").replace(',', ".")
    } else if t.matches('.').count() == 1 && t.rsplit('.').next().is_some_and(|d| d.len() <= 2) {
        t
    } else {
        t.replace('.', "")
    };
    let d: Decimal = normalized.parse().ok()?;
    (d * Decimal::from(100)).round().to_i64()
}

fn date(s: &str) -> Option<NaiveDate> {
    let t = s.trim();
    NaiveDate::parse_from_str(t, "%d/%m/%Y")
        .or_else(|_| NaiveDate::parse_from_str(t, "%Y-%m-%d"))
        .or_else(|_| NaiveDate::parse_from_str(t, "%d/%m/%y"))
        .ok()
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Col {
    Date,
    Total,
    Amortization,
    Interest,
    Insurance,
    Fees,
    Balance,
    Adjustment,
    Paid,
    Number,
    Skip,
}

fn column(header: &str) -> Col {
    let h = crate::import::text::fold(header);
    if h.starts_with("data") {
        Col::Date
    } else if h.contains("amortiza") {
        Col::Amortization
    } else if h.contains("juros") {
        Col::Interest
    } else if h.contains("seguro") {
        Col::Insurance
    } else if h.contains("taxa") || h.contains("tarifa") {
        Col::Fees
    } else if h.contains("saldo") {
        Col::Balance
    } else if h.contains("correc") || h.contains("atualiza") {
        Col::Adjustment
    } else if h.contains("pago") {
        Col::Paid
    } else if h.starts_with("n da")
        || h.starts_with("no da")
        || h.starts_with("numero")
        || h == "n"
        || h == "no"
        || h == "parcela"
    {
        Col::Number
    } else if h.contains("prestac") || (h.contains("encargo") && !h.contains("liquid")) {
        Col::Total
    } else {
        Col::Skip
    }
}

/// Ordem das colunas da planilha da Caixa (sem cabeçalho): data, prestação, nº, amortização,
/// juros, seguros, taxas, encargos líquido, valor devido, saldo, correção, valor pago.
const CAIXA: [Col; 12] = [
    Col::Date,
    Col::Total,
    Col::Number,
    Col::Amortization,
    Col::Interest,
    Col::Insurance,
    Col::Fees,
    Col::Skip,
    Col::Skip,
    Col::Balance,
    Col::Adjustment,
    Col::Paid,
];

fn cells(line: &str) -> Vec<&str> {
    let parts: Vec<&str> = if line.contains('\t') {
        line.split('\t').collect()
    } else if line.contains(';') {
        line.split(';').collect()
    } else {
        // Copiado de PDF/tela: colunas separadas por dois ou mais espaços.
        line.split("  ").filter(|s| !s.trim().is_empty()).collect()
    };
    parts.into_iter().map(str::trim).collect()
}

/// Lê o texto colado: cabeçalho pelos nomes das colunas ou a ordem da planilha da Caixa.
pub fn parse(text: &str) -> (Vec<StatementRow>, Vec<String>) {
    let mut cols: Option<Vec<Col>> = None;
    let mut rows = vec![];
    let mut bad = vec![];
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let c = cells(line);
        if cols.is_none() && c.first().is_some_and(|f| date(f).is_none()) {
            let mapped: Vec<Col> = c.iter().map(|h| column(h)).collect();
            if mapped.contains(&Col::Date) && mapped.contains(&Col::Balance) {
                cols = Some(mapped);
            }
            continue;
        }
        let map: Vec<Col> = cols.clone().unwrap_or_else(|| CAIXA.to_vec());
        let get = |want: Col| -> Option<&str> {
            map.iter()
                .position(|k| *k == want)
                .and_then(|i| c.get(i).copied())
        };
        let parsed = (|| {
            let d = date(get(Col::Date)?)?;
            let amortization = money(get(Col::Amortization)?)?;
            let interest = money(get(Col::Interest)?)?;
            let balance_after = money(get(Col::Balance)?)?;
            let insurance = get(Col::Insurance).and_then(money).unwrap_or(0);
            let fees = get(Col::Fees).and_then(money).unwrap_or(0);
            let total = get(Col::Total)
                .and_then(money)
                .unwrap_or(amortization + interest + insurance + fees);
            Some(StatementRow {
                date: fmt_date(d),
                total,
                amortization,
                interest,
                insurance,
                fees,
                adjustment: get(Col::Adjustment).and_then(money).unwrap_or(0),
                balance_after,
                paid: get(Col::Paid).and_then(money),
                number: get(Col::Number).and_then(|n| n.trim().parse().ok()),
            })
        })();
        match parsed {
            Some(r) => rows.push(r),
            None => bad.push(line.chars().take(80).collect::<String>()),
        }
    }
    rows.sort_by(|a, b| a.date.cmp(&b.date));
    rows.dedup_by(|a, b| a.date == b.date);
    let warnings = if bad.is_empty() {
        vec![]
    } else {
        vec![t_args(
            "nucleo-planejamento-divida-extrato-linhas-nao-entendidas",
            &args(&[("n", bad.len().into()), ("exemplo", bad[0].clone().into())]),
        )]
    };
    (rows, warnings)
}

fn dec(v: i64) -> Decimal {
    Decimal::from(v)
}

/// Prazo que falta depois da linha (Price: pela fórmula da prestação; SAC: saldo ÷
/// amortização).
fn remaining_after(system: AmortSystem, r: &StatementRow) -> Option<u32> {
    let before = dec(r.balance_after + r.amortization);
    if before <= Decimal::ZERO || r.amortization <= 0 {
        return None;
    }
    let n = match system {
        AmortSystem::Sac => (before / dec(r.amortization)).to_f64()?,
        _ => {
            let i = (dec(r.interest) / before).to_f64()?;
            let pmt = (r.amortization + r.interest) as f64;
            let x = 1.0 - i * before.to_f64()? / pmt;
            if i <= 0.0 || x <= 0.0 {
                return None;
            }
            -x.ln() / (1.0 + i).ln()
        }
    };
    let n = n.round();
    (1.0..1000.0).contains(&n).then(|| n as u32 - 1)
}

fn pct(v: Decimal, dp: u32) -> String {
    v.round_dp(dp).normalize().to_string()
}

/// [`pct`] com o separador decimal do idioma, para os avisos.
fn pct_label(v: Decimal, dp: u32) -> String {
    crate::ai::alerts::decimal(i18n::current(), &pct(v, dp))
}

/// Analisa o extrato contra o contrato (sem gravar).
pub fn preview(conn: &Connection, a: &StatementArgs) -> Result<StatementPreview> {
    let (system, rate): (String, String) = conn
        .query_row(
            "SELECT system, annual_rate FROM loan WHERE id = ?1",
            [&a.id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| Error::not_found(t("nucleo-planejamento-divida-contrato")))?;
    let system: AmortSystem = parse_kind(&system)?;
    let (mut rows, mut warnings) = match (&a.rows, &a.text) {
        (Some(r), _) if !r.is_empty() => (r.clone(), vec![]),
        (_, Some(t)) => parse(t),
        _ => {
            return Err(Error::invalid(t(
                "nucleo-planejamento-divida-extrato-erro-vazio",
            )));
        }
    };
    rows.sort_by(|x, y| x.date.cmp(&y.date));
    if rows.is_empty() {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-extrato-erro-nenhuma-linha",
        )));
    }
    // Amortizações extraordinárias: o saldo caiu mais do que a amortização da parcela.
    let mut extras = vec![];
    for w in rows.windows(2) {
        let (prev, cur) = (&w[0], &w[1]);
        let expected = prev.balance_after + cur.adjustment - cur.amortization;
        let extra = expected - cur.balance_after;
        // O banco recomeça a numeração na versão nova do contrato (depois da amortização).
        let new_version = cur.number == Some(1) && prev.number.is_some_and(|n| n > 1);
        if extra > 5_000.max(cur.total / 2) || (new_version && extra > 0) {
            extras.push(InferredExtra {
                date: cur.date.clone(),
                amount: extra,
            });
        }
    }
    // Taxa implícita: juros ÷ saldo antes da amortização, média das últimas linhas.
    let recent: Vec<Decimal> = rows
        .iter()
        .rev()
        .take(6)
        .filter(|r| r.balance_after + r.amortization > 0 && r.interest > 0)
        .map(|r| dec(r.interest) / dec(r.balance_after + r.amortization))
        .collect();
    let monthly = (!recent.is_empty())
        .then(|| recent.iter().sum::<Decimal>() / Decimal::from(recent.len() as u32));
    let contract: Decimal = rate.parse().unwrap_or_default();
    let effective = monthly.map(|m| {
        (crate::finance::pow_dec(Decimal::ONE + m, Decimal::from(12)) - Decimal::ONE)
            * Decimal::ONE_HUNDRED
    });
    let mut rate_mismatch = false;
    if let (Some(m), Some(e)) = (monthly, effective) {
        let contract_m = schedule::monthly_rate(contract);
        if (m - contract_m).abs() > Decimal::from_f64(0.00002).unwrap_or_default() {
            rate_mismatch = true;
            warnings.push(t_args(
                "nucleo-planejamento-divida-extrato-aviso-juros",
                &args(&[
                    ("mensal", pct_label(m * Decimal::ONE_HUNDRED, 4).into()),
                    ("nominal", pct_label(m * Decimal::from(1200), 2).into()),
                    ("efetiva", pct_label(e, 4).into()),
                    ("contrato", pct_label(contract, 4).into()),
                ]),
            ));
        }
    }
    let last = rows.last().expect("não vazio");
    let remaining = a.remaining_term.or_else(|| remaining_after(system, last));
    if remaining.is_none() {
        warnings.push(t("nucleo-planejamento-divida-extrato-aviso-prazo"));
    }
    Ok(StatementPreview {
        first_date: rows.first().map(|r| r.date.clone()),
        last_date: Some(last.date.clone()),
        last_balance: Some(last.balance_after),
        remaining_term: remaining,
        extras,
        implied_monthly_pct: monthly.map(|m| pct(m * Decimal::ONE_HUNDRED, 4)),
        implied_nominal_annual_pct: monthly.map(|m| pct(m * Decimal::from(1200), 2)),
        implied_effective_annual_pct: effective.map(|e| pct(e, 4)),
        contract_rate_pct: pct(contract, 6),
        rows,
        warnings,
        rate_mismatch,
    })
}

/// Número da parcela do mês da data (1 = mês do primeiro vencimento).
pub(crate) fn number_for(first_due: NaiveDate, d: NaiveDate) -> Option<u32> {
    let k = (d.year() - first_due.year()) * 12 + d.month() as i32 - first_due.month() as i32 + 1;
    (k >= 1).then_some(k as u32)
}

/// Grava o extrato: parcelas reais pagas, as anteriores marcadas como pagas e a projeção a
/// partir do último saldo.
pub fn import(tx: &Transaction<'_>, a: &StatementArgs) -> Result<StatementResult> {
    let p = preview(tx, a)?;
    let Some(term) = p.remaining_term else {
        return Err(Error::invalid(t(
            "nucleo-planejamento-divida-extrato-erro-prazo",
        )));
    };
    let first_due: String = tx.query_row(
        "SELECT first_due_date FROM loan WHERE id = ?1",
        [&a.id],
        |r| r.get(0),
    )?;
    let first_due = crate::time::parse_date(&first_due)?;
    let now = now_utc();
    let mut imported = 0;
    let mut last_number = 0;
    for r in &p.rows {
        let d = crate::time::parse_date(&r.date)?;
        let Some(n) = number_for(first_due, d) else {
            let l = i18n::current();
            return Err(Error::invalid(t_args(
                "nucleo-planejamento-divida-extrato-erro-antes-do-vencimento",
                &args(&[
                    ("data", i18n::date(l, d).into()),
                    ("vencimento", i18n::date(l, first_due).into()),
                ]),
            )));
        };
        let extra = p
            .extras
            .iter()
            .find(|e| e.date == r.date)
            .map_or(0, |e| e.amount);
        let existing: Option<String> = tx
            .query_row(
                "SELECT id FROM loan_installment WHERE loan_id = ?1 AND number = ?2",
                params![a.id, n],
                |x| x.get(0),
            )
            .optional()?;
        let paid = r.paid.unwrap_or(r.total);
        match existing {
            Some(iid) => {
                tx.execute(
                    "UPDATE loan_installment SET due_date = ?2, amortization = ?3, interest = ?4, adjustment = ?5,
                            insurance = ?6, fees = ?7, total = ?8, balance_after = ?9, estimated = 0,
                            extra_payment = ?10, status = 'paid', paid_date = coalesce(paid_date, ?2),
                            paid_amount = ?11, source = 'statement', insurance_mip = 0, insurance_dfi = 0,
                            updated_at = ?12
                      WHERE id = ?1",
                    params![
                        iid,
                        r.date,
                        r.amortization,
                        r.interest,
                        r.adjustment,
                        r.insurance,
                        r.fees,
                        r.total,
                        r.balance_after,
                        extra,
                        paid,
                        now
                    ],
                )?;
            }
            None => {
                tx.execute(
                    "INSERT INTO loan_installment (id, loan_id, number, due_date, amortization, interest, adjustment, insurance,
                                                   fees, total, balance_after, estimated, extra_payment, status, paid_date,
                                                   paid_amount, source, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, ?12, 'paid', ?4, ?13, 'statement', ?14, ?14)",
                    params![
                        new_id(),
                        a.id,
                        n,
                        r.date,
                        r.amortization,
                        r.interest,
                        r.adjustment,
                        r.insurance,
                        r.fees,
                        r.total,
                        r.balance_after,
                        extra,
                        paid,
                        now
                    ],
                )?;
            }
        }
        imported += 1;
        last_number = last_number.max(n);
    }
    // Parcelas antes do extrato, em aberto: pagas no vencimento (o contrato está em dia).
    let mut marked = 0;
    if a.mark_previous_paid.unwrap_or(true) {
        let first = p.rows.first().map(|r| r.date.clone()).unwrap_or_default();
        marked = tx.execute(
            "UPDATE loan_installment SET status = 'paid', paid_date = due_date, paid_amount = total, updated_at = ?3
              WHERE loan_id = ?1 AND status <> 'paid' AND due_date < ?2",
            params![a.id, first, now],
        )? as u32;
    }
    // Parcelas do cronograma antigo entre as do extrato (meses que faltaram) ficam como estão.
    tx.execute(
        "UPDATE loan SET rebase_number = ?2, rebase_balance = ?3, rebase_term = ?4, updated_at = ?5 WHERE id = ?1",
        params![a.id, last_number, p.last_balance, term, now],
    )?;
    regenerate(tx, &a.id)?;
    Ok(StatementResult {
        preview: StatementPreview {
            remaining_term: Some(term),
            ..p
        },
        imported,
        marked_previous: marked,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_caixa_sheet_with_and_without_header() {
        let header = "Data do pagamento\tValor da prestação\tN da prestação\tAmortização\tJuros\tSeguros\tTaxas\tEncargos líquido\tValor devido\tSaldo devedor\tCorreção monetária\tValor pago";
        let rows = "02/07/2026\tR$ 3.962,03\t26\tR$ 1.089,95\tR$ 2.742,83\tR$ 104,25\tR$ 25,00\tR$ 3.962,03\tR$ 3.962,03\tR$ 346.542,14\tR$ 592,62\tR$ 3.962,05\n\
                    02/08/2026\tR$ 3.953,31\t1\tR$ 1.205,18\tR$ 2.621,19\tR$ 101,94\tR$ 25,00\tR$ 3.953,31\tR$ 3.953,31\tR$ 331.009,50\tR$ 154,88\tR$ 3.953,31";
        for text in [format!("{header}\n{rows}"), rows.to_string()] {
            let (r, w) = parse(&text);
            assert!(w.is_empty(), "{w:?}");
            assert_eq!(r.len(), 2);
            assert_eq!(r[1].date, "2026-08-02");
            assert_eq!(r[1].amortization, 120_518);
            assert_eq!(r[1].interest, 262_119);
            assert_eq!(r[1].balance_after, 33_100_950);
            assert_eq!(r[1].adjustment, 15_488);
            assert_eq!(r[0].paid, Some(396_205));
        }
    }

    #[test]
    fn remaining_term_from_the_price_installment() {
        let r = StatementRow {
            date: "2026-10-02".into(),
            total: 399_862,
            amortization: 122_843,
            interest: 261_096,
            insurance: 13_423,
            fees: 2_500,
            adjustment: 55_787,
            balance_after: 32_968_852,
            paid: None,
            number: Some(3),
        };
        // Bank: term 147 after 02/08/2026 → 144 after 02/10/2026.
        assert_eq!(remaining_after(AmortSystem::Price, &r), Some(144));
    }
}
