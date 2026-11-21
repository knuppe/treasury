//! SEC EDGAR (gratuito, exige User-Agent identificado com e-mail de contato):
//! `company_tickers.json` (ticker → CIK) e a API XBRL `companyfacts` (fatos us-gaap/dei de
//! cada companhia). Só parse de bytes.
//!
//! Cada conceito tem vários candidatos em ordem de preferência (as companhias trocam de
//! conceito ao longo dos anos: `Revenues` → `RevenueFromContractWithCustomer…`); para cada
//! período vale o primeiro candidato que o informa. Fatos repetidos em vários formulários
//! ficam com o arquivado por último. Durações de ~12 meses são exercícios; as que começam no
//! início de um exercício são acumulados (10-Q); trimestres isolados de 10-Q são ignorados.

use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;

use super::{Fact, PeriodType, items};
use crate::time::add_months;

pub const TICKERS_URL: &str = "https://www.sec.gov/files/company_tickers.json";

pub fn facts_url(cik: u64) -> String {
    format!("https://data.sec.gov/api/xbrl/companyfacts/CIK{cik:010}.json")
}

/// Id do emissor para um CIK.
/// Cadastro da empresa na SEC (setor pelo SIC).
pub fn submissions_url(cik: u64) -> String {
    format!("https://data.sec.gov/submissions/CIK{cik:010}.json")
}

/// SIC (código e descrição) do cadastro da SEC.
pub fn parse_submissions(body: &[u8]) -> Result<(Option<String>, Option<String>), String> {
    let v: serde_json::Value = serde_json::from_slice(body).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-pesquisa-erro-sec-cadastro",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    let text = |k: &str| {
        v.get(k)
            .and_then(|x| x.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };
    Ok((text("sic"), text("sicDescription")))
}

pub fn issuer_id(cik: u64) -> String {
    format!("cik:{cik:010}")
}

/// Ticker no padrão da SEC (`BRK.B` → `BRK-B`).
pub fn normalize_ticker(t: &str) -> String {
    t.trim().to_uppercase().replace('.', "-")
}

#[derive(Debug, Clone, PartialEq)]
pub struct TickerEntry {
    pub cik: u64,
    pub ticker: String,
    pub title: String,
}

/// `{"0":{"cik_str":320193,"ticker":"AAPL","title":"Apple Inc."}, …}` → ticker → entrada.
pub fn parse_company_tickers(body: &[u8]) -> Result<HashMap<String, TickerEntry>, String> {
    #[derive(Deserialize)]
    struct Row {
        cik_str: u64,
        ticker: String,
        title: String,
    }
    let rows: HashMap<String, Row> = serde_json::from_slice(body).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-pesquisa-erro-sec-tickers",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    Ok(rows
        .into_values()
        .map(|r| {
            let t = normalize_ticker(&r.ticker);
            (
                t.clone(),
                TickerEntry {
                    cik: r.cik_str,
                    ticker: t,
                    title: r.title,
                },
            )
        })
        .collect())
}

#[derive(Deserialize)]
struct CompanyFacts {
    #[serde(rename = "entityName")]
    entity_name: Option<String>,
    facts: HashMap<String, HashMap<String, Concept>>,
}

#[derive(Deserialize)]
struct Concept {
    units: HashMap<String, Vec<Row>>,
}

#[derive(Deserialize, Clone)]
struct Row {
    start: Option<String>,
    end: String,
    val: serde_json::Number,
    accn: Option<String>,
    form: Option<String>,
    filed: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Flow,
    Instant,
}

struct Mapping {
    item: &'static str,
    kind: Kind,
    unit: &'static str,
    /// −1 para guardar saídas de caixa como negativas (capex).
    sign: i64,
    concepts: &'static [(&'static str, &'static str)],
}

const G: &str = "us-gaap";

const MAPPINGS: &[Mapping] = &[
    Mapping {
        item: items::REVENUE,
        kind: Kind::Flow,
        unit: "USD",
        sign: 1,
        concepts: &[
            (G, "Revenues"),
            (G, "RevenueFromContractWithCustomerExcludingAssessedTax"),
            (G, "RevenueFromContractWithCustomerIncludingAssessedTax"),
            (G, "SalesRevenueNet"),
        ],
    },
    Mapping {
        item: items::GROSS_PROFIT,
        kind: Kind::Flow,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "GrossProfit")],
    },
    Mapping {
        item: items::EBIT,
        kind: Kind::Flow,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "OperatingIncomeLoss")],
    },
    Mapping {
        item: items::DA,
        kind: Kind::Flow,
        unit: "USD",
        sign: 1,
        concepts: &[
            (G, "DepreciationDepletionAndAmortization"),
            (G, "DepreciationAmortizationAndAccretionNet"),
            (G, "DepreciationAndAmortization"),
            (G, "Depreciation"),
        ],
    },
    Mapping {
        item: items::NET_INCOME,
        kind: Kind::Flow,
        unit: "USD",
        sign: 1,
        concepts: &[
            (G, "NetIncomeLoss"),
            (G, "NetIncomeLossAvailableToCommonStockholdersBasic"),
            (G, "ProfitLoss"),
        ],
    },
    Mapping {
        item: items::CFO,
        kind: Kind::Flow,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "NetCashProvidedByUsedInOperatingActivities")],
    },
    Mapping {
        item: items::CAPEX,
        kind: Kind::Flow,
        unit: "USD",
        sign: -1,
        concepts: &[(G, "PaymentsToAcquirePropertyPlantAndEquipment")],
    },
    Mapping {
        item: items::DIVIDENDS_PAID,
        kind: Kind::Flow,
        unit: "USD",
        sign: 1,
        concepts: &[
            (G, "PaymentsOfDividends"),
            (G, "PaymentsOfDividendsCommonStock"),
        ],
    },
    Mapping {
        item: items::DIVIDENDS_DECLARED,
        kind: Kind::Flow,
        unit: "USD",
        sign: 1,
        concepts: &[
            (G, "DividendsCommonStock"),
            (G, "Dividends"),
            (G, "DividendsCommonStockCash"),
        ],
    },
    Mapping {
        item: items::DPS_DECLARED,
        kind: Kind::Flow,
        unit: "USD/shares",
        sign: 1,
        concepts: &[
            (G, "CommonStockDividendsPerShareDeclared"),
            (G, "CommonStockDividendsPerShareCashPaid"),
        ],
    },
    Mapping {
        item: items::TOTAL_ASSETS,
        kind: Kind::Instant,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "Assets")],
    },
    Mapping {
        item: items::CURRENT_ASSETS,
        kind: Kind::Instant,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "AssetsCurrent")],
    },
    Mapping {
        item: items::CURRENT_LIABILITIES,
        kind: Kind::Instant,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "LiabilitiesCurrent")],
    },
    Mapping {
        item: items::TOTAL_LIABILITIES,
        kind: Kind::Instant,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "Liabilities")],
    },
    Mapping {
        item: items::EQUITY,
        kind: Kind::Instant,
        unit: "USD",
        sign: 1,
        concepts: &[
            (G, "StockholdersEquity"),
            (
                G,
                "StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest",
            ),
        ],
    },
    Mapping {
        item: items::SHARES,
        kind: Kind::Instant,
        unit: "shares",
        sign: 1,
        concepts: &[
            ("dei", "EntityCommonStockSharesOutstanding"),
            (G, "CommonStockSharesOutstanding"),
        ],
    },
    Mapping {
        item: items::COGS,
        kind: Kind::Flow,
        unit: "USD",
        sign: 1,
        concepts: &[
            (G, "CostOfRevenue"),
            (G, "CostOfGoodsAndServicesSold"),
            (G, "CostOfGoodsSold"),
        ],
    },
    Mapping {
        item: items::SGA,
        kind: Kind::Flow,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "SellingGeneralAndAdministrativeExpense")],
    },
    Mapping {
        item: items::FIN_EXPENSE,
        kind: Kind::Flow,
        unit: "USD",
        sign: 1,
        concepts: &[
            (G, "InterestExpense"),
            (G, "InterestExpenseNonoperating"),
            (G, "InterestExpenseDebt"),
            (G, "InterestExpenseOperating"),
        ],
    },
    Mapping {
        item: items::FIN_INCOME,
        kind: Kind::Flow,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "InvestmentIncomeInterest")],
    },
    Mapping {
        item: items::EBT,
        kind: Kind::Flow,
        unit: "USD",
        sign: 1,
        concepts: &[
            (
                G,
                "IncomeLossFromContinuingOperationsBeforeIncomeTaxesExtraordinaryItemsNoncontrollingInterest",
            ),
            (
                G,
                "IncomeLossFromContinuingOperationsBeforeIncomeTaxesMinorityInterestAndIncomeLossFromEquityMethodInvestments",
            ),
        ],
    },
    Mapping {
        item: items::INCOME_TAX,
        kind: Kind::Flow,
        unit: "USD",
        sign: -1,
        concepts: &[(G, "IncomeTaxExpenseBenefit")],
    },
    Mapping {
        item: items::BUYBACKS,
        kind: Kind::Flow,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "PaymentsForRepurchaseOfCommonStock")],
    },
    Mapping {
        item: items::SHARE_ISSUANCE,
        kind: Kind::Flow,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "ProceedsFromIssuanceOfCommonStock")],
    },
    Mapping {
        item: items::RECEIVABLES,
        kind: Kind::Instant,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "AccountsReceivableNetCurrent")],
    },
    Mapping {
        item: items::INVENTORIES,
        kind: Kind::Instant,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "InventoryNet")],
    },
    Mapping {
        item: items::PPE,
        kind: Kind::Instant,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "PropertyPlantAndEquipmentNet")],
    },
    Mapping {
        item: items::INTANGIBLE,
        kind: Kind::Instant,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "IntangibleAssetsNetExcludingGoodwill")],
    },
    Mapping {
        item: items::SUPPLIERS,
        kind: Kind::Instant,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "AccountsPayableCurrent")],
    },
    Mapping {
        item: items::DEBT_ST,
        kind: Kind::Instant,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "LongTermDebtCurrent"), (G, "DebtCurrent")],
    },
    Mapping {
        item: items::DEBT_LT,
        kind: Kind::Instant,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "LongTermDebtNoncurrent")],
    },
    Mapping {
        item: items::LEASES,
        kind: Kind::Instant,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "OperatingLeaseLiability")],
    },
    Mapping {
        item: items::RETAINED,
        kind: Kind::Instant,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "RetainedEarningsAccumulatedDeficit")],
    },
    Mapping {
        item: items::MINORITY,
        kind: Kind::Instant,
        unit: "USD",
        sign: 1,
        concepts: &[(G, "MinorityInterest")],
    },
];

/// Partes da dívida e do caixa, somadas por data.
/// Dívida total: o primeiro conceito informado na data (REITs costumam usar notas a pagar).
const DEBT_TOTAL: &[&str] = &[
    "LongTermDebt",
    "NotesPayable",
    "SeniorNotes",
    "UnsecuredDebt",
    // Só notas de longo prazo ou conversíveis (a GameStop, nos trimestres de 2026).
    "LongTermNotesPayable",
    "ConvertibleDebt",
    "ConvertibleNotesPayable",
    "ConvertibleLongTermNotesPayable",
];
const DEBT_NONCURRENT: &str = "LongTermDebtNoncurrent";
const DEBT_CURRENT: &[&str] = &["LongTermDebtCurrent", "DebtCurrent"];
const DEBT_SHORT: &[&str] = &["CommercialPaper", "ShortTermBorrowings"];
const CASH: &[&str] = &[
    "CashAndCashEquivalentsAtCarryingValue",
    "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
];
const SHORT_INV: &[&str] = &[
    "ShortTermInvestments",
    "MarketableSecuritiesCurrent",
    "AvailableForSaleSecuritiesDebtSecuritiesCurrent",
];

#[derive(Clone)]
struct Point {
    start: Option<NaiveDate>,
    end: NaiveDate,
    value: Decimal,
    document: String,
    filed: String,
}

fn dt(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// Pontos de um conceito, um por período (o arquivado por último vence).
fn points(
    cf: &CompanyFacts,
    ns: &str,
    concept: &str,
    unit: &str,
) -> BTreeMap<(Option<NaiveDate>, NaiveDate), Point> {
    let mut out: BTreeMap<(Option<NaiveDate>, NaiveDate), Point> = BTreeMap::new();
    let Some(rows) = cf
        .facts
        .get(ns)
        .and_then(|m| m.get(concept))
        .and_then(|c| c.units.get(unit))
    else {
        return out;
    };
    for r in rows {
        let (Some(end), Ok(value)) = (
            dt(&r.end),
            r.val
                .to_string()
                .parse::<Decimal>()
                .or_else(|_| Decimal::from_scientific(&r.val.to_string())),
        ) else {
            continue;
        };
        let start = r.start.as_deref().and_then(dt);
        let filed = r.filed.clone().unwrap_or_default();
        let document = crate::i18n::t_args(
            "nucleo-pesquisa-doc-sec",
            &crate::i18n::args(&[
                (
                    "formulario",
                    r.form.as_deref().unwrap_or("SEC").to_string().into(),
                ),
                ("numero", r.accn.as_deref().unwrap_or("").to_string().into()),
                ("data", filed.clone().into()),
            ]),
        );
        let p = Point {
            start,
            end,
            value,
            document,
            filed,
        };
        match out.get(&(start, end)) {
            Some(old) if old.filed >= p.filed => {}
            _ => {
                out.insert((start, end), p);
            }
        }
    }
    out
}

/// Fatos us-gaap de uma companhia (`companyfacts`). Devolve o nome e os fatos.
pub fn parse_companyfacts(body: &[u8]) -> Result<(String, Vec<Fact>), String> {
    let cf: CompanyFacts = serde_json::from_slice(body).map_err(|e| {
        crate::i18n::t_args(
            "nucleo-pesquisa-erro-sec-companyfacts",
            &crate::i18n::args(&[("erro", e.to_string().into())]),
        )
    })?;
    let name = cf.entity_name.clone().unwrap_or_default();
    // Início dos exercícios: pelo lucro (ou receita) anual.
    let mut fy_starts: HashSet<NaiveDate> = HashSet::new();
    for m in MAPPINGS
        .iter()
        .filter(|m| m.item == items::NET_INCOME || m.item == items::REVENUE)
    {
        for (ns, c) in m.concepts {
            for p in points(&cf, ns, c, m.unit).values() {
                if let Some(s) = p.start {
                    let d = (p.end - s).num_days();
                    if (350..=380).contains(&d) {
                        fy_starts.insert(s);
                        fy_starts.insert(p.end.succ_opt().expect("data válida"));
                    }
                }
            }
        }
    }
    let is_fy_start = |s: NaiveDate| fy_starts.iter().any(|f| super::periods::near(*f, s));

    let mut facts: Vec<Fact> = vec![];
    let mut push = |item: &str, kind: Kind, unit: &str, p: &Point, value: Decimal| {
        let pt = match kind {
            Kind::Instant => PeriodType::Instant,
            Kind::Flow => {
                let Some(s) = p.start else { return };
                let d = (p.end - s).num_days();
                if (350..=380).contains(&d) {
                    PeriodType::Annual
                } else if d < 350 && is_fy_start(s) {
                    PeriodType::Ytd
                } else {
                    return;
                }
            }
        };
        facts.push(Fact {
            item: item.into(),
            period_type: pt,
            start: if kind == Kind::Flow { p.start } else { None },
            end: p.end,
            value,
            currency: match unit {
                "USD/shares" | "USD" => "USD".into(),
                other => other.into(),
            },
            source: "sec_companyfacts".into(),
            document: p.document.clone(),
            priority: 2,
        });
    };
    for m in MAPPINGS {
        let mut seen: HashSet<(Option<NaiveDate>, NaiveDate)> = HashSet::new();
        for (ns, c) in m.concepts {
            for (k, p) in points(&cf, ns, c, m.unit) {
                let key = if m.kind == Kind::Instant {
                    (None, k.1)
                } else {
                    k
                };
                if seen.insert(key) {
                    push(m.item, m.kind, m.unit, &p, p.value * Decimal::from(m.sign));
                }
            }
        }
    }
    // Dívida bruta e caixa: soma das partes na mesma data.
    let sum_at = |names: &[&str]| -> BTreeMap<NaiveDate, Point> {
        let mut out: BTreeMap<NaiveDate, Point> = BTreeMap::new();
        for n in names {
            for ((_, end), p) in points(&cf, G, n, "USD") {
                out.entry(end).or_insert(p);
            }
        }
        out
    };
    let total = sum_at(DEBT_TOTAL);
    let noncurrent = sum_at(&[DEBT_NONCURRENT]);
    let current = sum_at(DEBT_CURRENT);
    let short = sum_at(DEBT_SHORT);
    let mut dates: Vec<NaiveDate> = total.keys().chain(noncurrent.keys()).copied().collect();
    dates.sort();
    dates.dedup();
    for d in dates {
        let base = match (total.get(&d), noncurrent.get(&d)) {
            (Some(t), _) => Some((t.value, t)),
            (None, Some(n)) => Some((
                n.value + current.get(&d).map(|c| c.value).unwrap_or_default(),
                n,
            )),
            _ => None,
        };
        if let Some((v, p)) = base {
            let v = v + short.get(&d).map(|s| s.value).unwrap_or_default();
            push(items::GROSS_DEBT, Kind::Instant, "USD", p, v);
        }
    }
    let cash = sum_at(CASH);
    let inv = sum_at(SHORT_INV);
    for (d, p) in &cash {
        let v = p.value + inv.get(d).map(|i| i.value).unwrap_or_default();
        push(items::CASH, Kind::Instant, "USD", p, v);
    }
    // Sem dívida nenhuma: balanço com caixa e passivo na data, mas sem conceito de dívida
    // (empresa sem empréstimos). Dívida bruta zero, para a dívida líquida virar caixa líquido.
    let liabilities = sum_at(&["Liabilities", "LiabilitiesCurrent"]);
    for (d, p) in &cash {
        let any_debt = [&total, &noncurrent, &current, &short]
            .iter()
            .any(|m| m.contains_key(d));
        if !any_debt && liabilities.contains_key(d) {
            push(items::GROSS_DEBT, Kind::Instant, "USD", p, Decimal::ZERO);
        }
    }
    // Só o necessário: 11 exercícios e os acumulados/saldos dos últimos 27 meses.
    let latest = facts.iter().map(|f| f.end).max();
    if let Some(latest) = latest {
        let annual_cut = add_months(latest, -12 * 11);
        let recent_cut = add_months(latest, -27);
        let annual_ends: HashSet<NaiveDate> = facts
            .iter()
            .filter(|f| f.period_type == PeriodType::Annual)
            .map(|f| f.end)
            .collect();
        facts.retain(|f| match f.period_type {
            PeriodType::Annual => f.end >= annual_cut,
            PeriodType::Instant if annual_ends.iter().any(|a| super::periods::near(*a, f.end)) => {
                f.end >= annual_cut
            }
            _ => f.end >= recent_cut,
        });
    }
    facts.sort_by(|a, b| (&a.item, a.period_type, a.end).cmp(&(&b.item, b.period_type, b.end)));
    Ok((name, facts))
}

#[cfg(test)]
pub(crate) mod fixtures {
    /// `companyfacts` inventado (formato real, poucos conceitos).
    pub fn companyfacts() -> Vec<u8> {
        let f = |start: &str, end: &str, val: &str, form: &str, filed: &str| {
            let s = if start.is_empty() {
                String::new()
            } else {
                format!("\"start\":\"{start}\",")
            };
            format!(
                "{{{s}\"end\":\"{end}\",\"val\":{val},\"accn\":\"0000000001-{filed}\",\"fy\":2026,\"fp\":\"FY\",\"form\":\"{form}\",\"filed\":\"{filed}\"}}"
            )
        };
        let concept = |name: &str, unit: &str, rows: Vec<String>| {
            format!(
                "\"{name}\":{{\"label\":\"x\",\"description\":\"x\",\"units\":{{\"{unit}\":[{}]}}}}",
                rows.join(",")
            )
        };
        let gaap = [
            concept(
                "Revenues",
                "USD",
                vec![f("2023-01-01", "2023-12-31", "800", "10-K", "2024-02-01")],
            ),
            concept(
                "RevenueFromContractWithCustomerExcludingAssessedTax",
                "USD",
                vec![
                    f("2023-01-01", "2023-12-31", "805", "10-K", "2024-02-01"),
                    f("2024-01-01", "2024-12-31", "1000", "10-K", "2025-02-01"),
                    f("2025-01-01", "2025-12-31", "1200", "10-K", "2026-02-01"),
                    f("2025-01-01", "2025-06-30", "550", "10-Q", "2025-08-01"),
                    f("2025-04-01", "2025-06-30", "290", "10-Q", "2025-08-01"),
                    f("2026-01-01", "2026-06-30", "700", "10-Q", "2026-08-01"),
                    f("2026-04-01", "2026-06-30", "360", "10-Q", "2026-08-01"),
                    // Comparativo repetido no 10-Q seguinte: fica o arquivado por último.
                    f("2025-01-01", "2025-06-30", "560", "10-Q", "2026-08-01"),
                ],
            ),
            concept(
                "NetIncomeLoss",
                "USD",
                vec![
                    f("2025-01-01", "2025-12-31", "240", "10-K", "2026-02-01"),
                    f("2025-01-01", "2025-06-30", "100", "10-Q", "2026-08-01"),
                    f("2026-01-01", "2026-06-30", "130", "10-Q", "2026-08-01"),
                ],
            ),
            concept(
                "PaymentsToAcquirePropertyPlantAndEquipment",
                "USD",
                vec![f("2025-01-01", "2025-12-31", "90", "10-K", "2026-02-01")],
            ),
            concept(
                "StockholdersEquity",
                "USD",
                vec![
                    f("", "2025-12-31", "2000", "10-K", "2026-02-01"),
                    f("", "2026-06-30", "2100", "10-Q", "2026-08-01"),
                ],
            ),
            concept(
                "LongTermDebtNoncurrent",
                "USD",
                vec![f("", "2026-06-30", "500", "10-Q", "2026-08-01")],
            ),
            concept(
                "LongTermDebtCurrent",
                "USD",
                vec![f("", "2026-06-30", "50", "10-Q", "2026-08-01")],
            ),
            concept(
                "CommercialPaper",
                "USD",
                vec![f("", "2026-06-30", "10", "10-Q", "2026-08-01")],
            ),
            concept(
                "CashAndCashEquivalentsAtCarryingValue",
                "USD",
                vec![f("", "2026-06-30", "300", "10-Q", "2026-08-01")],
            ),
            concept(
                "ShortTermInvestments",
                "USD",
                vec![f("", "2026-06-30", "20", "10-Q", "2026-08-01")],
            ),
            concept(
                "CommonStockDividendsPerShareDeclared",
                "USD/shares",
                vec![f("2025-01-01", "2025-12-31", "1.04", "10-K", "2026-02-01")],
            ),
        ];
        let dei = concept(
            "EntityCommonStockSharesOutstanding",
            "shares",
            vec![f("", "2026-07-20", "100", "10-Q", "2026-08-01")],
        );
        format!(
            "{{\"cik\":1234,\"entityName\":\"Example Corp\",\"facts\":{{\"dei\":{{{dei}}},\"us-gaap\":{{{}}}}}}}",
            gaap.join(",")
        )
        .into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn parses_tickers() {
        let m = parse_company_tickers(
            br#"{"0":{"cik_str":320193,"ticker":"AAPL","title":"Apple Inc."},"1":{"cik_str":1067983,"ticker":"BRK-B","title":"Berkshire"}}"#,
        )
        .unwrap();
        assert_eq!(m["AAPL"].cik, 320193);
        assert_eq!(m[&normalize_ticker("brk.b")].cik, 1067983);
        assert_eq!(
            facts_url(320193),
            "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json"
        );
        assert!(parse_company_tickers(b"[]").is_err());
    }

    #[test]
    fn parses_companyfacts_with_candidates_and_dedup() {
        let (name, facts) = parse_companyfacts(&fixtures::companyfacts()).unwrap();
        assert_eq!(name, "Example Corp");
        let get = |item: &str, pt: PeriodType, end: &str| {
            facts
                .iter()
                .find(|f| f.item == item && f.period_type == pt && f.end == d(end))
                .map(|f| f.value)
        };
        // 2023: `Revenues` vem antes na preferência.
        assert_eq!(
            get(items::REVENUE, PeriodType::Annual, "2023-12-31"),
            Some(dec!(800))
        );
        assert_eq!(
            get(items::REVENUE, PeriodType::Annual, "2025-12-31"),
            Some(dec!(1200))
        );
        // Acumulado de 6 meses, com o comparativo reapresentado (arquivado por último).
        assert_eq!(
            get(items::REVENUE, PeriodType::Ytd, "2025-06-30"),
            Some(dec!(560))
        );
        // O trimestre isolado (abril–junho) não vira acumulado.
        assert_eq!(
            facts
                .iter()
                .filter(|f| f.item == items::REVENUE && f.period_type == PeriodType::Ytd)
                .count(),
            2
        );
        assert_eq!(
            get(items::CAPEX, PeriodType::Annual, "2025-12-31"),
            Some(dec!(-90))
        );
        assert_eq!(
            get(items::GROSS_DEBT, PeriodType::Instant, "2026-06-30"),
            Some(dec!(560))
        );
        assert_eq!(
            get(items::CASH, PeriodType::Instant, "2026-06-30"),
            Some(dec!(320))
        );
        assert_eq!(
            get(items::SHARES, PeriodType::Instant, "2026-07-20"),
            Some(dec!(100))
        );
        assert_eq!(
            get(items::DPS_DECLARED, PeriodType::Annual, "2025-12-31"),
            Some(dec!(1.04))
        );
        let ltm = super::super::periods::ltm_at(&facts, items::REVENUE, d("2026-06-30")).unwrap();
        assert_eq!(ltm.value, dec!(1340), "1200 + 700 − 560");
        assert!(facts.iter().all(|f| f.source == "sec_companyfacts"));
        assert!(
            facts
                .iter()
                .find(|f| f.item == items::SHARES)
                .unwrap()
                .currency
                == "shares"
        );
    }

    #[test]
    fn company_without_any_debt_concept_gets_zero_gross_debt() {
        let row = |end: &str, val: &str| {
            format!(
                "{{\"end\":\"{end}\",\"val\":{val},\"accn\":\"0000000001-26\",\"fy\":2026,\"fp\":\"Q2\",\"form\":\"10-Q\",\"filed\":\"2026-08-01\"}}"
            )
        };
        let c = |name: &str, rows: Vec<String>| {
            format!(
                "\"{name}\":{{\"label\":\"x\",\"description\":\"x\",\"units\":{{\"USD\":[{}]}}}}",
                rows.join(",")
            )
        };
        let gaap = [
            c(
                "CashAndCashEquivalentsAtCarryingValue",
                vec![row("2026-06-30", "300"), row("2025-12-31", "250")],
            ),
            // Passivo só numa das datas: a outra fica sem dívida (não dá para afirmar zero).
            c("Liabilities", vec![row("2026-06-30", "120")]),
        ];
        let body = format!(
            "{{\"cik\":1,\"entityName\":\"Sem Dívida\",\"facts\":{{\"us-gaap\":{{{}}}}}}}",
            gaap.join(",")
        );
        let (_, facts) = parse_companyfacts(body.as_bytes()).unwrap();
        let debt: Vec<_> = facts
            .iter()
            .filter(|f| f.item == items::GROSS_DEBT)
            .map(|f| (f.end, f.value))
            .collect();
        assert_eq!(debt, vec![(d("2026-06-30"), dec!(0))]);
    }
}
