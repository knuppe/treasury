//! O que o Treasury percebe nos dados **sem IA** (docs/insights.md): regras, estatística
//! simples e dados oficiais, calculados no computador. Pensado para quem não tem acesso a
//! um modelo de IA: cada percepção traz os números, o porquê e onde agir.
//!
//! - **Inflação pessoal**: o peso de cada grupo do IPCA nos gastos da família (12 meses
//!   completos) no lugar do peso da cesta do IBGE, com a variação de cada grupo (IBGE SIDRA
//!   7060). Quem gasta mais com comida e moradia sente outra inflação.
//! - **Tarifas e juros pagos** nos últimos 12 meses (pelos textos do extrato e pelas
//!   categorias), com o lembrete dos serviços essenciais gratuitos e o custo do crédito.
//! - **Conta no vermelho**: dias com saldo negativo em conta corrente nos últimos 90 dias
//!   (cheque especial), comparando os juros dele com os de um crédito mais barato.
//! - **Gasto fora do normal**: categoria do último mês completo bem acima (ou abaixo) da
//!   mediana dos 6 meses anteriores.
//! - **Compras pequenas**: muitas compras de até R$ 30 que somam bastante no mês.
//! - **Gastos acima da renda**: nos últimos 3 meses completos.
//! - **Essencial × resto × sobra**: a divisão da renda dos últimos 3 meses (categorias
//!   marcadas como essenciais), comparada à referência 50/30/20.
//!
//! Os textos saem do catálogo (`locales/*/insights.ftl`) no idioma da instalação, com valores,
//! porcentagens e meses no formato dele.

#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDate;
use rusqlite::{Connection, params};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ts_rs::TS;

use crate::budget::Basis;
use crate::categories::CategoryKind;
use crate::error::Result;
use crate::i18n::{self, args, t_args};
use crate::import::text::fold;
use crate::reports::engine::{FlowQuery, Fx, Key, balances_at, month_dates, shift_month};

/// Compras até este valor contam como "pequenas".
pub const SMALL_PURCHASE: i64 = 3_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum InsightKind {
    PersonalInflation,
    FeesAndInterest,
    Overdraft,
    CategorySpike,
    CategoryDrop,
    SmallPurchases,
    SpendingAboveIncome,
    /// Essencial × resto × o que sobrou da renda (referência 50/30/20).
    BudgetSplit,
    /// O empregador não depositou o FGTS de uma competência no prazo (docs/brazil/fgts.md, 6.2).
    FgtsMissingDeposit,
    /// Histórico de pagamentos da previdência com pendências (o que os prints não fecharam)
    /// ou aportes esperando a cota da CVM há mais de uma semana.
    PensionPayments,
    /// Nenhuma forma de abrir o cofre sem a senha (kit, partes ou chave física), ou o kit
    /// ficou velho com a troca de senha.
    RecoveryKit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum InsightTone {
    /// Atenção (algo custando caro ou fora do normal).
    Warning,
    Info,
    /// Notícia boa (gasto caindo).
    Good,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Insight {
    /// Chave estável (para dispensar na interface).
    pub id: String,
    pub kind: InsightKind,
    pub tone: InsightTone,
    pub title: String,
    /// Os números e o porquê, em linguagem simples.
    pub body: String,
    /// O que fazer, em uma frase.
    pub action: Option<String>,
    /// Tela onde agir (`orcamento`, `dividas`, `lancamentos`…).
    pub screen: Option<String>,
    /// Valor principal em centavos (na moeda da família), quando houver.
    #[ts(type = "number | null")]
    pub amount: Option<i64>,
    #[ts(type = "Record<string, unknown>")]
    pub data: serde_json::Value,
}

/// Um grupo do IPCA na inflação pessoal.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InflationGroup {
    pub id: String,
    pub name: String,
    /// Fração dos gastos da família no grupo (0–1).
    pub weight: f64,
    /// Variação do grupo em 12 meses (fração).
    pub change: f64,
    /// Contribuição para a inflação pessoal (peso × variação).
    pub contribution: f64,
    #[ts(type = "number")]
    pub spent: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PersonalInflation {
    pub from_month: String,
    pub to_month: String,
    /// Inflação pessoal e o IPCA no mesmo período (frações).
    pub personal: f64,
    pub ipca: Option<f64>,
    pub groups: Vec<InflationGroup>,
    /// Fração dos gastos que entrou na conta (o resto não tem grupo no IPCA: impostos,
    /// tarifas, dívidas, investimentos…).
    pub coverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Insights {
    pub items: Vec<Insight>,
    pub personal_inflation: Option<PersonalInflation>,
    pub currency: String,
}

/// Grupos do IPCA (séries `ipca_g1`…`ipca_g9`) e as palavras das categorias de cada um,
/// procuradas primeiro na subcategoria e depois na categoria-pai.
pub const GROUPS: &[(&str, &str, &[&str])] = &[
    (
        "ipca_g9",
        "Comunicação",
        &[
            "internet",
            "telefone",
            "celular",
            "comunicacao",
            "streaming",
            "tv por assinatura",
        ],
    ),
    (
        "ipca_g8",
        "Educação",
        &[
            "educacao",
            "escola",
            "faculdade",
            "curso",
            "livro",
            "material escolar",
            "mensalidade escolar",
        ],
    ),
    (
        "ipca_g1",
        "Alimentação e bebidas",
        &[
            "alimentacao",
            "mercado",
            "supermercado",
            "restaurante",
            "padaria",
            "delivery",
            "lanche",
            "feira",
            "acougue",
            "hortifruti",
            "bebida",
        ],
    ),
    (
        "ipca_g2",
        "Habitação",
        &[
            "moradia",
            "habitacao",
            "aluguel",
            "condominio",
            "energia",
            "luz",
            "agua",
            "gas",
            "iptu",
            "saneamento",
        ],
    ),
    (
        "ipca_g3",
        "Artigos de residência",
        &[
            "moveis",
            "eletrodomestico",
            "eletronico",
            "decoracao",
            "utilidades domesticas",
            "casa e decoracao",
        ],
    ),
    (
        "ipca_g4",
        "Vestuário",
        &["vestuario", "roupa", "calcado", "acessorio"],
    ),
    (
        "ipca_g5",
        "Transportes",
        &[
            "transporte",
            "combustivel",
            "uber",
            "aplicativo",
            "onibus",
            "metro",
            "estacionamento",
            "pedagio",
            "veiculo",
            "carro",
            "moto",
            "ipva",
            "manutencao do carro",
        ],
    ),
    (
        "ipca_g6",
        "Saúde e cuidados pessoais",
        &[
            "saude",
            "farmacia",
            "remedio",
            "medico",
            "dentista",
            "plano de saude",
            "exame",
            "higiene",
            "cuidados pessoais",
            "beleza",
            "cabeleireiro",
        ],
    ),
    (
        "ipca_g7",
        "Despesas pessoais",
        &[
            "lazer",
            "viagem",
            "cinema",
            "show",
            "recreacao",
            "academia",
            "pet",
            "presente",
            "servicos pessoais",
            "hobby",
            "festa",
            "cigarro",
        ],
    ),
];

/// Grupo do IPCA de uma categoria ("Pai › Filho"); `None` = fora do IPCA.
pub fn group_of(path: &str) -> Option<&'static str> {
    let parts: Vec<String> = path.split('›').map(|p| fold(p.trim())).collect();
    for part in parts.iter().rev() {
        for (id, _, words) in GROUPS {
            if words.iter().any(|w| contains_word(part, w)) {
                return Some(id);
            }
        }
    }
    None
}

fn contains_word(text: &str, w: &str) -> bool {
    let t = format!(
        " {} ",
        text.split(|c: char| !c.is_alphanumeric())
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    );
    t.contains(&format!(" {w} ")) || (w.len() > 5 && t.contains(&format!(" {w}")))
}

fn base_currency(conn: &Connection) -> String {
    crate::household::base_currency(conn).unwrap_or_else(|_| "BRL".into())
}

fn money(v: i64, cur: &str) -> String {
    i18n::money(i18n::current(), v, cur)
}

fn pct(v: f64) -> String {
    crate::ai::alerts::pct(i18n::current(), v, 1)
}

/// Porcentagem sem a casa decimal quando ela é zero (50% em vez de 50,0%).
fn pct_short(v: f64) -> String {
    let l = i18n::current();
    pct(v).replace(&crate::ai::alerts::decimal(l, ".0%"), "%")
}

/// Mês `AAAA-MM` abreviado (out/2026, Oct 2026).
fn month_label(m: &str) -> String {
    crate::ai::alerts::month_short(i18n::current(), m)
}

/// Nome do grupo do IPCA no idioma da instalação.
fn group_name(id: &str) -> String {
    i18n::t(&format!("insights-grupo-{}", id.replace('_', "-")))
}

fn last_complete_month(today: NaiveDate) -> String {
    let m = crate::time::month_of(today);
    shift_month(&m, -1).unwrap_or(m)
}

/// Gastos por categoria (id) em cada mês, na moeda da família.
fn expenses_by_month(
    conn: &Connection,
    cur: &str,
    from: &str,
    to: &str,
) -> Result<BTreeMap<(String, String), i64>> {
    let (start, _) = month_dates(from)?;
    let (_, end) = month_dates(to)?;
    let mut q = FlowQuery::new(&start, &end, Basis::Accrual);
    q.kind = Some(CategoryKind::Expense);
    let mut fx = Fx::new(conn, cur);
    let agg = q.aggregate(conn, &mut fx, Key::Month, Key::Category)?;
    Ok(agg
        .cells
        .into_iter()
        .filter_map(|((m, c), cell)| Some(((m?, c.unwrap_or_default()), cell.expense)))
        .collect())
}

fn totals_by_month(
    conn: &Connection,
    cur: &str,
    from: &str,
    to: &str,
) -> Result<BTreeMap<String, (i64, i64)>> {
    let (start, _) = month_dates(from)?;
    let (_, end) = month_dates(to)?;
    let q = FlowQuery::new(&start, &end, Basis::Accrual);
    let mut fx = Fx::new(conn, cur);
    let agg = q.aggregate(conn, &mut fx, Key::Month, Key::None)?;
    Ok(agg
        .cells
        .into_iter()
        .filter_map(|((m, _), c)| Some((m?, (c.income, c.expense))))
        .collect())
}

/// Variação acumulada de uma série mensal (% a.m.) nos meses dados.
fn accumulated(conn: &Connection, series: &str, months: &[String]) -> Result<Option<f64>> {
    let mut acc = 1.0f64;
    let mut found = 0;
    let mut stmt = conn.prepare_cached(
        "SELECT value FROM index_value WHERE series_id = ?1 AND substr(date, 1, 7) = ?2 AND estimated = 0",
    )?;
    for m in months {
        let v: Option<String> = stmt.query_row(params![series, m], |r| r.get(0)).ok();
        if let Some(v) = v
            .and_then(|s| s.parse::<Decimal>().ok())
            .and_then(|d| d.to_f64())
        {
            acc *= 1.0 + v / 100.0;
            found += 1;
        }
    }
    // Precisa de quase todos os meses (o IBGE publica com um mês de atraso).
    Ok((found + 1 >= months.len() && found > 0).then_some(acc - 1.0))
}

/// Inflação pessoal dos últimos 12 meses com variação publicada.
pub fn personal_inflation(
    conn: &Connection,
    today: NaiveDate,
) -> Result<Option<PersonalInflation>> {
    let cur = base_currency(conn);
    let to = last_complete_month(today);
    let from = shift_month(&to, -11)?;
    let months = crate::reports::engine::months_between(&from, &to)?;
    // O grupo sai do caminho em pt-BR: vale com as categorias em qualquer idioma.
    let paths: HashMap<String, String> = crate::categories::br_categories(conn)?
        .into_iter()
        .map(|c| (c.id, c.path))
        .collect();
    let spent = expenses_by_month(conn, &cur, &from, &to)?;
    let mut by_group: HashMap<&'static str, i64> = HashMap::new();
    let mut total = 0i64;
    for ((_, cat), v) in &spent {
        if *v <= 0 {
            continue;
        }
        total += v;
        if let Some(g) = paths.get(cat).and_then(|p| group_of(p)) {
            *by_group.entry(g).or_default() += v;
        }
    }
    let mapped: i64 = by_group.values().sum();
    if total <= 0 || mapped * 2 < total {
        return Ok(None);
    }
    let mut groups = vec![];
    let mut personal = 0.0;
    let mut weight_with_data = 0.0;
    for (id, _, _) in GROUPS {
        let s = by_group.get(id).copied().unwrap_or(0);
        if s <= 0 {
            continue;
        }
        let Some(change) = accumulated(conn, id, &months)? else {
            continue;
        };
        let w = s as f64 / mapped as f64;
        weight_with_data += w;
        groups.push(InflationGroup {
            id: id.to_string(),
            name: group_name(id),
            weight: w,
            change,
            contribution: w * change,
            spent: s,
        });
        personal += w * change;
    }
    if groups.is_empty() || weight_with_data < 0.5 {
        return Ok(None);
    }
    // Renormaliza os pesos aos grupos com dado.
    personal /= weight_with_data;
    for g in &mut groups {
        g.weight /= weight_with_data;
        g.contribution /= weight_with_data;
    }
    groups.sort_by(|a, b| b.contribution.total_cmp(&a.contribution));
    Ok(Some(PersonalInflation {
        from_month: from,
        to_month: to,
        personal,
        ipca: accumulated(conn, "ipca", &months)?,
        groups,
        coverage: mapped as f64 / total as f64,
    }))
}

/// Padrões de tarifa e de juros nos textos do extrato.
const FEE_WORDS: &[&str] = &[
    "tarifa",
    "tar ",
    "pacote de servicos",
    "cesta de servicos",
    "anuidade",
    "manutencao de conta",
    "taxa de manutencao",
    "mensalidade pacote",
];
const INTEREST_WORDS: &[&str] = &[
    "juros",
    "encargos",
    "iof",
    "mora",
    "multa por atraso",
    "cheque especial",
    "limite da conta",
    "rotativo",
    "encargo",
];

fn matches_any(text: &str, words: &[&str]) -> bool {
    let t = format!(" {} ", fold(text));
    words
        .iter()
        .any(|w| t.contains(&format!(" {w}")) || (w.ends_with(' ') && t.contains(w)))
}

fn fees_and_interest(conn: &Connection, today: NaiveDate, cur: &str) -> Result<Option<Insight>> {
    let to = last_complete_month(today);
    let from = shift_month(&to, -11)?;
    let (start, _) = month_dates(&from)?;
    let (_, end) = month_dates(&to)?;
    let paths = crate::categories::path_map(conn)?;
    let mut stmt = conn.prepare(
        "SELECT t.description, t.amount, t.currency, t.date, t.category_id FROM txn t
          WHERE t.amount < 0 AND t.status IN ('cleared', 'reconciled') AND t.kind IN ('expense', 'transfer')
            AND t.transfer_id IS NULL AND t.date BETWEEN ?1 AND ?2",
    )?;
    let rows: Vec<(String, i64, String, String, Option<String>)> = stmt
        .query_map(params![start, end], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let mut fx = Fx::new(conn, cur);
    let (mut fees, mut interest, mut nf, mut ni) = (0i64, 0i64, 0i64, 0i64);
    for (desc, amount, currency, date, cat) in rows {
        let path = cat
            .as_ref()
            .and_then(|c| paths.get(c))
            .map(|p| fold(p))
            .unwrap_or_default();
        let is_fee = matches_any(&desc, FEE_WORDS) || path.contains("tarifa");
        let is_interest = matches_any(&desc, INTEREST_WORDS) || path.contains("juros");
        if !is_fee && !is_interest {
            continue;
        }
        let Some(c) = fx.convert(-amount, &currency, &date)? else {
            continue;
        };
        if is_interest {
            interest += c.amount;
            ni += 1;
        } else {
            fees += c.amount;
            nf += 1;
        }
    }
    let total = fees + interest;
    if total < 5_000 {
        return Ok(None);
    }
    let mut body = t_args(
        "insights-tarifas-corpo",
        &args(&[
            ("de", month_label(&from).into()),
            ("ate", month_label(&to).into()),
            ("tarifas", money(fees, cur).into()),
            ("n_tarifas", nf.into()),
            ("juros", money(interest, cur).into()),
            ("n_juros", ni.into()),
        ]),
    );
    if interest > 0 {
        body.push(' ');
        body.push_str(&i18n::t("insights-tarifas-juros-caros"));
    }
    let action = match (fees > 0, interest > 0) {
        (true, true) => "insights-tarifas-acao-ambos",
        (true, false) => "insights-tarifas-acao-pacote",
        _ => "insights-tarifas-acao-credito",
    };
    Ok(Some(Insight {
        id: format!("fees:{to}"),
        kind: InsightKind::FeesAndInterest,
        tone: InsightTone::Warning,
        title: t_args(
            "insights-tarifas-titulo",
            &args(&[("valor", money(total, cur).into())]),
        ),
        body,
        action: Some(i18n::t(action)),
        screen: Some("lancamentos".into()),
        amount: Some(total),
        data: json!({"fees": fees, "interest": interest, "fee_count": nf, "interest_count": ni}),
    }))
}

fn overdraft(conn: &Connection, today: NaiveDate) -> Result<Vec<Insight>> {
    let start = today - chrono::Duration::days(90);
    let before = crate::time::fmt_date(start - chrono::Duration::days(1));
    let accounts: Vec<_> = balances_at(conn, &before)?
        .into_iter()
        .filter(|a| a.kind == crate::accounts::AccountKind::Checking && !a.archived)
        .collect();
    let mut out = vec![];
    for a in accounts {
        let spell = crate::accounts::overdraft::negative_spell(conn, &a.id, start, today)?;
        let (neg_days, min) = (spell.days, spell.min);
        if neg_days < 3 || min >= 0 {
            continue;
        }
        let rate = overdraft_median(conn)?;
        let rate_txt = rate
            .map(|r| {
                format!(
                    " {}",
                    t_args(
                        "insights-vermelho-taxa",
                        &args(&[(
                            "taxa",
                            crate::ai::alerts::decimal(i18n::current(), &format!("{r:.1}")).into(),
                        )]),
                    )
                )
            })
            .unwrap_or_default();
        let a_args = args(&[
            ("conta", a.name.clone().into()),
            ("dias", neg_days.into()),
            ("minimo", money(min, &a.currency).into()),
        ]);
        out.push(Insight {
            id: format!("overdraft:{}:{}", a.id, crate::time::month_of(today)),
            kind: InsightKind::Overdraft,
            tone: InsightTone::Warning,
            title: t_args("insights-vermelho-titulo", &a_args),
            body: t_args("insights-vermelho-corpo", &a_args) + &rate_txt,
            action: Some(i18n::t("insights-vermelho-acao")),
            screen: Some("dividas".into()),
            amount: Some(min),
            data: json!({"account_id": a.id, "negative_days": neg_days, "min_balance": min}),
        });
    }
    Ok(out)
}

/// Competências sem depósito do FGTS depois do prazo, dentro do período coberto pelo último
/// extrato importado: um alerta por conta, a partir da primeira competência em falta.
fn fgts_missing(conn: &Connection, today: NaiveDate) -> Result<Vec<Insight>> {
    if !crate::country::is_brazil(conn) {
        return Ok(vec![]);
    }
    let mut out = vec![];
    for a in crate::fgts::list(conn, today)? {
        let (Some(last), true, false) = (&a.last_statement_date, a.active, a.archived) else {
            continue;
        };
        let check = crate::fgts::check(conn, &a.account_id, today)?;
        let missing: Vec<&crate::fgts::DepositCheck> = check
            .deposits
            .iter()
            .filter(|d| d.status == "missing" && d.due_date.as_str() <= last.as_str())
            .collect();
        let Some(first) = missing.first() else {
            continue;
        };
        let a_args = args(&[
            ("conta", a.name.clone().into()),
            (
                "mes",
                crate::reports::engine::month_label(&first.competence).into(),
            ),
            ("n", missing.len().into()),
        ]);
        out.push(Insight {
            id: format!("fgts-missing:{}:{}", a.account_id, first.competence),
            kind: InsightKind::FgtsMissingDeposit,
            tone: InsightTone::Warning,
            title: t_args("insights-fgts-titulo", &a_args),
            body: t_args("insights-fgts-corpo", &a_args),
            action: Some(i18n::t("insights-fgts-acao")),
            screen: Some("contas".into()),
            amount: first.payslip,
            data: json!({"account_id": a.account_id, "competences": missing.iter().map(|d| &d.competence).collect::<Vec<_>>()}),
        });
    }
    Ok(out)
}

/// Planos de previdência com pendências na importação do histórico de pagamentos, ou com
/// aportes esperando a cota da CVM há mais de 7 dias: um alerta por plano.
fn pension_payments(conn: &Connection, today: NaiveDate) -> Result<Vec<Insight>> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.name FROM asset a WHERE a.class = 'pension' AND a.archived = 0
            AND (EXISTS (SELECT 1 FROM pension_payment_issue i WHERE i.asset_id = a.id)
                 OR EXISTS (SELECT 1 FROM pension_payment p WHERE p.asset_id = a.id
                             AND p.trade_id IS NULL AND p.created_at < ?1))
          ORDER BY a.name",
    )?;
    let week_ago = crate::time::fmt_date(today - chrono::Duration::days(7));
    let plans: Vec<(String, String)> = stmt
        .query_map([&week_ago], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = vec![];
    for (id, name) in plans {
        let issues = crate::pension::payments::saved_issues(conn, &id)?;
        let (pending, amount, since) = crate::pension::payments::pending_summary(conn, &id)?;
        let a = args(&[
            ("plano", name.clone().into()),
            ("n", issues.len().into()),
            ("aportes", pending.into()),
            (
                "desde",
                since
                    .as_deref()
                    .and_then(|d| crate::time::parse_date(d).ok())
                    .map(|d| i18n::date(i18n::current(), d))
                    .unwrap_or_default()
                    .into(),
            ),
        ]);
        let mut body = vec![];
        if !issues.is_empty() {
            body.push(t_args("insights-previdencia-pendencias", &a));
            body.extend(issues.iter().take(5).map(|i| format!("• {}", i.message)));
        }
        if pending > 0 {
            body.push(t_args("insights-previdencia-sem-cota", &a));
        }
        out.push(Insight {
            id: format!("pension-payments:{id}:{}:{pending}", issues.len()),
            kind: InsightKind::PensionPayments,
            tone: InsightTone::Warning,
            title: t_args("insights-previdencia-titulo", &a),
            body: body.join("\n"),
            action: Some(i18n::t("insights-previdencia-acao")),
            screen: Some(format!("ativos?ativo={id}&secao=previdencia")),
            amount: (amount > 0).then_some(amount),
            data: json!({"asset_id": id, "issues": issues, "pending_payments": pending, "since": since}),
        });
    }
    Ok(out)
}

/// Mediana dos bancos no cheque especial (% a.m.), se as taxas do BCB foram baixadas.
fn overdraft_median(conn: &Connection) -> Result<Option<f64>> {
    let mut stmt = conn.prepare_cached(
        "SELECT rate_month FROM credit_rate WHERE modality = 'Cheque especial - Prefixado'
           AND period_start = (SELECT max(period_start) FROM credit_rate WHERE modality = 'Cheque especial - Prefixado')
         ORDER BY rate_month",
    )?;
    let v: Vec<f64> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok((!v.is_empty()).then(|| v[v.len() / 2]))
}

fn median(v: &mut [i64]) -> i64 {
    if v.is_empty() {
        return 0;
    }
    v.sort_unstable();
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2
    }
}

fn category_changes(conn: &Connection, today: NaiveDate, cur: &str) -> Result<Vec<Insight>> {
    let last = last_complete_month(today);
    let from = shift_month(&last, -6)?;
    let spent = expenses_by_month(conn, cur, &from, &last)?;
    let paths = crate::categories::path_map(conn)?;
    // Por categoria-pai: o mês e os 6 anteriores.
    let root = |cat: &str| -> String {
        paths
            .get(cat)
            .map(|p| p.split('›').next().unwrap_or(p).trim().to_string())
            .unwrap_or_else(|| "Sem categoria".into())
    };
    let mut by: BTreeMap<String, BTreeMap<String, i64>> = BTreeMap::new();
    for ((m, c), v) in &spent {
        *by.entry(root(c)).or_default().entry(m.clone()).or_default() += v;
    }
    let prior: Vec<String> = (1..=6)
        .filter_map(|i| shift_month(&last, -i).ok())
        .collect();
    let mut ups = vec![];
    let mut downs = vec![];
    for (cat, months) in by {
        if cat == "Sem categoria" {
            continue;
        }
        let now = months.get(&last).copied().unwrap_or(0);
        let mut base: Vec<i64> = prior
            .iter()
            .map(|m| months.get(m).copied().unwrap_or(0))
            .collect();
        let present = base.iter().filter(|v| **v > 0).count();
        if present < 4 {
            continue;
        }
        let med = median(&mut base);
        if med < 10_000 {
            continue;
        }
        let diff = now - med;
        if now as f64 >= med as f64 * 1.4 && diff >= 10_000 {
            ups.push((diff, cat, now, med));
        } else if (now as f64) <= med as f64 * 0.7 && -diff >= 10_000 {
            downs.push((-diff, cat, now, med));
        }
    }
    ups.sort_by_key(|x| std::cmp::Reverse(x.0));
    downs.sort_by_key(|x| std::cmp::Reverse(x.0));
    let label = month_label(&last);
    let change_args = |cat: &str, now: i64, diff: i64, med: i64| {
        args(&[
            ("categoria", cat.to_string().into()),
            ("mes", label.clone().into()),
            ("valor", money(now, cur).into()),
            ("diferenca", money(diff, cur).into()),
            ("normal", money(med, cur).into()),
        ])
    };
    let mut out = vec![];
    for (diff, cat, now, med) in ups.into_iter().take(3) {
        let a = change_args(&cat, now, diff, med);
        out.push(Insight {
            id: format!("spike:{last}:{cat}"),
            kind: InsightKind::CategorySpike,
            tone: InsightTone::Warning,
            title: t_args("insights-acima-titulo", &a),
            body: t_args("insights-acima-corpo", &a),
            action: Some(i18n::t("insights-acima-acao")),
            screen: Some("lancamentos".into()),
            amount: Some(diff),
            data: json!({"category": cat, "month": last, "amount": now, "normal": med}),
        });
    }
    for (diff, cat, now, med) in downs.into_iter().take(2) {
        let a = change_args(&cat, now, diff, med);
        out.push(Insight {
            id: format!("drop:{last}:{cat}"),
            kind: InsightKind::CategoryDrop,
            tone: InsightTone::Good,
            title: t_args("insights-abaixo-titulo", &a),
            body: t_args("insights-abaixo-corpo", &a),
            action: Some(i18n::t("insights-abaixo-acao")),
            screen: Some("orcamento".into()),
            amount: Some(diff),
            data: json!({"category": cat, "month": last, "amount": now, "normal": med}),
        });
    }
    Ok(out)
}

fn small_purchases(conn: &Connection, today: NaiveDate, cur: &str) -> Result<Option<Insight>> {
    let last = last_complete_month(today);
    let (start, end) = month_dates(&last)?;
    let paths = crate::categories::path_map(conn)?;
    let mut stmt = conn.prepare(
        "SELECT amount, category_id FROM txn WHERE kind = 'expense' AND status IN ('cleared', 'reconciled')
          AND currency = ?1 AND amount < 0 AND -amount <= ?2 AND date BETWEEN ?3 AND ?4",
    )?;
    let rows: Vec<(i64, Option<String>)> = stmt
        .query_map(params![cur, SMALL_PURCHASE, start, end], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    let total: i64 = rows.iter().map(|r| -r.0).sum();
    if rows.len() < 15 || total < 20_000 {
        return Ok(None);
    }
    let mut by: HashMap<String, i64> = HashMap::new();
    for (a, c) in &rows {
        let name = c
            .as_ref()
            .and_then(|c| paths.get(c))
            .map(|p| p.split('›').next_back().unwrap_or(p).trim().to_string())
            .unwrap_or_else(|| i18n::t("insights-sem-categoria"));
        *by.entry(name).or_default() -= a;
    }
    let mut top: Vec<(String, i64)> = by.into_iter().collect();
    top.sort_by_key(|x| std::cmp::Reverse(x.1));
    let names: Vec<String> = top
        .iter()
        .take(3)
        .map(|(n, v)| {
            t_args(
                "insights-pequenas-item",
                &args(&[
                    ("categoria", n.clone().into()),
                    ("valor", money(*v, cur).into()),
                ]),
            )
        })
        .collect();
    let a = args(&[
        ("n", (rows.len() as i64).into()),
        ("total", money(total, cur).into()),
        ("mes", month_label(&last).into()),
        ("limite", money(SMALL_PURCHASE, cur).into()),
        ("lista", names.join(", ").into()),
    ]);
    Ok(Some(Insight {
        id: format!("small:{last}"),
        kind: InsightKind::SmallPurchases,
        tone: InsightTone::Info,
        title: t_args("insights-pequenas-titulo", &a),
        body: t_args("insights-pequenas-corpo", &a),
        action: Some(i18n::t("insights-pequenas-acao")),
        screen: Some("orcamento".into()),
        amount: Some(total),
        data: json!({"count": rows.len(), "total": total, "month": last}),
    }))
}

fn spending_above_income(
    conn: &Connection,
    today: NaiveDate,
    cur: &str,
) -> Result<Option<Insight>> {
    let to = last_complete_month(today);
    let from = shift_month(&to, -2)?;
    let t = totals_by_month(conn, cur, &from, &to)?;
    if t.len() < 3 {
        return Ok(None);
    }
    let income: i64 = t.values().map(|v| v.0).sum();
    let expense: i64 = t.values().map(|v| v.1).sum();
    if income <= 0 || expense <= income {
        return Ok(None);
    }
    let gap = expense - income;
    if gap < 10_000 {
        return Ok(None);
    }
    Ok(Some(Insight {
        id: format!("deficit:{to}"),
        kind: InsightKind::SpendingAboveIncome,
        tone: InsightTone::Warning,
        title: t_args(
            "insights-deficit-titulo",
            &args(&[("valor", money(gap, cur).into())]),
        ),
        body: t_args(
            "insights-deficit-corpo",
            &args(&[
                ("de", month_label(&from).into()),
                ("ate", month_label(&to).into()),
                ("renda", money(income, cur).into()),
                ("despesas", money(expense, cur).into()),
            ]),
        ),
        action: Some(i18n::t("insights-deficit-acao")),
        screen: Some("orcamento".into()),
        amount: Some(gap),
        data: json!({"income": income, "expense": expense}),
    }))
}

fn budget_split(conn: &Connection, today: NaiveDate, cur: &str) -> Result<Option<Insight>> {
    let to = last_complete_month(today);
    let from = shift_month(&to, -2)?;
    let t = totals_by_month(conn, cur, &from, &to)?;
    if t.len() < 3 {
        return Ok(None);
    }
    let income: i64 = t.values().map(|v| v.0).sum();
    let expense: i64 = t.values().map(|v| v.1).sum();
    if income <= 0 || expense > income {
        return Ok(None);
    }
    let tree = crate::reports::engine::CategoryTree::load(conn)?;
    let spent = expenses_by_month(conn, cur, &from, &to)?;
    let essential: i64 = spent
        .iter()
        .filter(|((_, c), _)| tree.get(c).is_some_and(|c| c.is_essential))
        .map(|(_, v)| *v)
        .sum();
    let other = expense - essential;
    let saved = income - expense;
    let share = |v: i64| v as f64 / income as f64;
    let (e, o, s) = (share(essential), share(other), share(saved));
    let tone = if s >= 0.2 {
        InsightTone::Good
    } else if e > 0.6 || s < 0.1 {
        InsightTone::Warning
    } else {
        InsightTone::Info
    };
    Ok(Some(Insight {
        id: format!("split:{to}"),
        kind: InsightKind::BudgetSplit,
        tone,
        title: t_args(
            "insights-divisao-titulo",
            &args(&[
                ("essencial", pct_short(e).into()),
                ("resto", pct_short(o).into()),
                ("sobra", pct_short(s).into()),
            ]),
        ),
        body: t_args(
            "insights-divisao-corpo",
            &args(&[
                ("de", month_label(&from).into()),
                ("ate", month_label(&to).into()),
                ("renda", money(income, cur).into()),
                ("essencial", money(essential, cur).into()),
                ("resto", money(other, cur).into()),
                ("sobra", money(saved, cur).into()),
            ]),
        ),
        action: Some(i18n::t(if s >= 0.2 {
            "insights-divisao-acao-folga"
        } else if e > 0.6 {
            "insights-divisao-acao-essencial"
        } else {
            "insights-divisao-acao-teto"
        })),
        screen: Some("orcamento".into()),
        amount: Some(saved),
        data: json!({"income": income, "essential": essential, "other": other, "saved": saved}),
    }))
}

/// Todas as percepções, as de atenção primeiro.
/// Sem kit de recuperação (nem partes nem chave física), esquecer a senha perde tudo. Volta
/// todo mês até a família gerar um (o id leva o mês, para não ficar dispensado para sempre).
/// O cofre de demonstração não tem senha de verdade e fica de fora.
fn recovery_kit(conn: &Connection, today: NaiveDate) -> Result<Option<Insight>> {
    if crate::settings::get::<bool>(conn, crate::demo::SETTING)?.unwrap_or(false) {
        return Ok(None);
    }
    let outdated =
        crate::settings::get::<bool>(conn, crate::unlock::KIT_OUTDATED)?.unwrap_or(false);
    if !outdated && crate::unlock::has_recovery(conn)? {
        return Ok(None);
    }
    let (title, body) = if outdated {
        ("insights-kit-velho-titulo", "insights-kit-velho-corpo")
    } else {
        ("insights-kit-titulo", "insights-kit-corpo")
    };
    Ok(Some(Insight {
        id: format!("recovery-kit:{}", fmt_month(today)),
        kind: InsightKind::RecoveryKit,
        tone: InsightTone::Warning,
        title: i18n::t(title),
        body: i18n::t(body),
        action: Some(i18n::t("insights-kit-acao")),
        screen: Some("ajustes?secao=seguranca".into()),
        amount: None,
        data: json!({ "outdated": outdated }),
    }))
}

fn fmt_month(d: NaiveDate) -> String {
    d.format("%Y-%m").to_string()
}

pub fn compute(conn: &Connection, today: NaiveDate) -> Result<Insights> {
    let cur = base_currency(conn);
    let mut items = vec![];
    items.extend(recovery_kit(conn, today)?);
    let pi = personal_inflation(conn, today)?;
    if let Some(p) = &pi {
        let top = p.groups.first();
        let diff = p.ipca.map(|i| p.personal - i);
        let tone = match diff {
            Some(d) if d > 0.005 => InsightTone::Warning,
            Some(d) if d < -0.005 => InsightTone::Good,
            _ => InsightTone::Info,
        };
        items.push(Insight {
            id: format!("inflation:{}", p.to_month),
            kind: InsightKind::PersonalInflation,
            tone,
            title: match p.ipca {
                Some(i) => t_args(
                    "insights-inflacao-titulo-ipca",
                    &args(&[("pessoal", pct(p.personal).into()), ("ipca", pct(i).into())]),
                ),
                None => t_args(
                    "insights-inflacao-titulo",
                    &args(&[("pessoal", pct(p.personal).into())]),
                ),
            },
            body: t_args(
                "insights-inflacao-corpo",
                &args(&[
                    ("pessoal", pct(p.personal).into()),
                    ("de", month_label(&p.from_month).into()),
                    ("ate", month_label(&p.to_month).into()),
                ]),
            ) + &top
                .map(|g| {
                    format!(
                        " {}",
                        t_args(
                            "insights-inflacao-mais-pesou",
                            &args(&[
                                ("grupo", g.name.clone().into()),
                                ("peso", pct(g.weight).into()),
                                ("variacao", pct(g.change).into()),
                            ]),
                        )
                    )
                })
                .unwrap_or_default(),
            action: Some(i18n::t("insights-inflacao-acao")),
            screen: Some("orcamento".into()),
            amount: None,
            data: serde_json::to_value(p)?,
        });
    }
    items.extend(fees_and_interest(conn, today, &cur)?);
    items.extend(overdraft(conn, today)?);
    items.extend(fgts_missing(conn, today)?);
    items.extend(pension_payments(conn, today)?);
    items.extend(spending_above_income(conn, today, &cur)?);
    items.extend(budget_split(conn, today, &cur)?);
    items.extend(category_changes(conn, today, &cur)?);
    items.extend(small_purchases(conn, today, &cur)?);
    items.sort_by_key(|i| i.tone);
    Ok(Insights {
        items,
        personal_inflation: pi,
        currency: cur,
    })
}
