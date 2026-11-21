//! Caixa de alertas (tabela `alert`): novo → visto → resolvido. Rotinas do gestor e o
//! agente criam; o usuário marca como visto ou resolvido.
//!
//! Idioma (V128, docs/i18n-design.md): o alerta guarda a chave do título (`title_key`) e os
//! argumentos (`args`, JSON `{"title": {…}, "body": [partes]}`), e o texto é montado no idioma
//! atual ao ler ([`list`], [`get`], [`created_since`]: interface, MCP e notificações). `title`
//! e `body` continuam gravados em pt-BR (reserva); as linhas de antes da V128 não têm chave e
//! aparecem como foram gravadas. Ver [`Msg`], [`Part`] e [`Arg`].

use std::borrow::Cow;
use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{self, Locale};
use crate::ids::{Id, new_id};
use crate::time::now_utc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AlertStatus {
    New,
    Seen,
    Resolved,
}

impl AlertStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            AlertStatus::New => "new",
            AlertStatus::Seen => "seen",
            AlertStatus::Resolved => "resolved",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl AlertSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            AlertSeverity::Info => "info",
            AlertSeverity::Warning => "warning",
            AlertSeverity::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Alert {
    pub id: Id,
    /// `budget_over`, `budget_pace`, `due_soon`, `rebalance`, `monthly_close`…
    pub kind: String,
    pub severity: AlertSeverity,
    pub title: String,
    /// Texto (Markdown simples).
    pub body: String,
    pub status: AlertStatus,
    /// Rotina ou motor que gerou.
    pub source: String,
    pub ref_table: Option<String>,
    pub ref_id: Option<String>,
    /// Números usados (valores em centavos).
    #[ts(type = "Record<string, unknown>")]
    pub data: Value,
    pub created_at: String,
    pub updated_at: String,
    pub seen_at: Option<String>,
    pub resolved_at: Option<String>,
    /// Nome do tipo no idioma de quem lê ("Saldo negativo previsto", "Score de saúde"…).
    pub kind_label: String,
    /// Onde abrir a origem do alerta no app (o ativo, a conta, a categoria…); `None` nos
    /// alertas genéricos.
    pub target: Option<AlertTarget>,
}

/// Tela de origem de um alerta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AlertScreen {
    /// Ativo cadastrado (Ativos › ativo).
    Asset,
    /// Conta (Contas › conta).
    Account,
    /// Categoria no orçamento do mês.
    Budget,
    /// Lançamentos de uma conta num dia (`account_id`, `date`).
    Transactions,
    /// Lançamentos de uma fatura de cartão (`id` = fatura).
    CardStatement,
    /// Bem (Carteira › Bens).
    PhysicalAsset,
    /// Alocação da carteira (rebalanceamento).
    Allocation,
    /// Relatórios do mês (`month`).
    Reports,
    /// Ajustes › Fontes de dados.
    Sources,
    /// Carteira › Saúde dos ativos › Dados dos ativos (sugestões das fontes).
    AssetData,
    /// Importar extrato (`account_id`: a conta sugerida).
    Import,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AlertTarget {
    pub screen: AlertScreen,
    pub id: Option<String>,
    pub account_id: Option<String>,
    /// Mês `YYYY-MM` (orçamento, relatórios).
    pub month: Option<String>,
    /// Dia `YYYY-MM-DD` (lançamentos).
    pub date: Option<String>,
}

impl AlertTarget {
    fn new(screen: AlertScreen, id: Option<String>) -> Self {
        Self {
            screen,
            id,
            account_id: None,
            month: None,
            date: None,
        }
    }
}

/// Tipo → chave do nome no catálogo (`alertas-tipo-*`); tipo desconhecido sai como veio.
fn kind_label(l: Locale, kind: &str) -> String {
    let key = format!("alertas-tipo-{}", kind.replace('_', "-"));
    if i18n::has_key(Locale::PtBr, &key) {
        i18n::tr(l, &key, None)
    } else {
        kind.to_string()
    }
}

/// Onde abrir a origem: pela referência gravada (ativo, conta, categoria, lançamento, fatura,
/// bem, parcela de IPTU) ou pelo tipo (fechamento, rebalanceamento, fonte desligada).
fn target_of(conn: &Connection, a: &Alert) -> Option<AlertTarget> {
    use AlertScreen as S;
    let id = a.ref_id.clone().filter(|s| !s.is_empty());
    let month = a
        .data
        .get("month")
        .and_then(Value::as_str)
        .map(str::to_string);
    let t = match (a.ref_table.as_deref(), id) {
        (Some("asset"), Some(id)) => AlertTarget::new(S::Asset, Some(id)),
        (Some("account"), Some(id)) => AlertTarget::new(S::Account, Some(id)),
        (Some("import"), account) => AlertTarget {
            account_id: account,
            ..AlertTarget::new(S::Import, None)
        },
        (Some("physical_asset"), Some(id)) => AlertTarget::new(S::PhysicalAsset, Some(id)),
        (Some("category"), Some(id)) => AlertTarget {
            month,
            ..AlertTarget::new(S::Budget, Some(id))
        },
        (Some("card_statement"), Some(id)) => {
            let account: Option<String> = conn
                .query_row(
                    "SELECT account_id FROM card_statement WHERE id = ?1",
                    [&id],
                    |r| r.get(0),
                )
                .optional()
                .ok()
                .flatten()?;
            AlertTarget {
                account_id: account,
                ..AlertTarget::new(S::CardStatement, Some(id))
            }
        }
        (Some("txn"), Some(id)) => {
            let (account, date): (String, String) = conn
                .query_row(
                    "SELECT account_id, date FROM txn WHERE id = ?1",
                    [&id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()
                .ok()
                .flatten()?;
            AlertTarget {
                account_id: Some(account),
                date: Some(date),
                ..AlertTarget::new(S::Transactions, Some(id))
            }
        }
        (Some("property_tax_installment"), Some(id)) => {
            let asset: String = conn
                .query_row(
                    "SELECT b.asset_id FROM property_tax_installment i
                       JOIN property_tax_bill b ON b.id = i.bill_id WHERE i.id = ?1",
                    [&id],
                    |r| r.get(0),
                )
                .optional()
                .ok()
                .flatten()?;
            AlertTarget::new(S::PhysicalAsset, Some(asset))
        }
        _ => match a.kind.as_str() {
            "monthly_close" => AlertTarget {
                month: a
                    .data
                    .pointer("/numbers/month")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(month),
                ..AlertTarget::new(S::Reports, None)
            },
            "rebalance" => AlertTarget::new(S::Allocation, None),
            "source_disabled" => AlertTarget::new(S::Sources, None),
            "asset_data_gaps" => AlertTarget::new(S::AssetData, None),
            _ => return None,
        },
    };
    Some(t)
}

/// Nome do tipo e destino, que dependem do banco e do idioma (não ficam gravados).
fn enrich(conn: &Connection, mut a: Alert) -> Alert {
    a.kind_label = kind_label(i18n::current(), &a.kind);
    a.target = target_of(conn, &a);
    a
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AlertListArgs {
    /// Padrão: novos e vistos.
    #[ts(optional)]
    pub status: Option<AlertStatus>,
    #[ts(optional)]
    pub kind: Option<String>,
    #[ts(optional, type = "number")]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AlertStatusArgs {
    pub ids: Vec<Id>,
    pub status: AlertStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AlertCounts {
    #[ts(type = "number")]
    pub new: i64,
    #[ts(type = "number")]
    pub seen: i64,
}

/// Alerta a criar. O título e o corpo vão pela chave do catálogo e são montados no idioma de
/// quem lê.
pub struct NewAlert<'a> {
    pub kind: &'a str,
    pub severity: AlertSeverity,
    pub title: Msg,
    pub body: Vec<Part>,
    pub source: &'a str,
    /// A mesma chave não gera dois alertas (mesmo depois de resolvido).
    pub dedupe_key: Option<String>,
    pub ref_table: Option<&'a str>,
    pub ref_id: Option<String>,
    pub data: Value,
}

/// Título em Markdown colado no fim do texto anterior ("…do mês.## Fechamento"), como alguns
/// motores entregam ao juntar as falas entre as ferramentas: volta para a própria linha, para a
/// tela desenhar o título (e a tabela que vem depois).
pub fn tidy_markdown(text: &str) -> String {
    use std::sync::OnceLock;
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(r"([^\n#|])[ \t]*(#{2,6} )(\p{Lu})").expect("regex dos títulos")
    });
    re.replace_all(text, "$1\n\n$2$3").into_owned()
}

/// Tira do texto as citações de ferramentas do MCP ("(get_budget_status)", "(fonte: a, b)",
/// "— get_monthly_flow", "segundo spending_summary,"): para a pessoa elas não dizem nada; os
/// nomes voltam à parte, para as fontes do alerta (em `data.sources`).
pub fn strip_tool_names(text: &str) -> (String, Vec<String>) {
    use std::sync::OnceLock;
    static RE: OnceLock<Vec<(regex::Regex, &'static str)>> = OnceLock::new();
    static NAME: OnceLock<regex::Regex> = OnceLock::new();
    let names = || {
        let mut n: Vec<&str> = crate::mcp::tools::TOOLS.iter().map(|t| t.name).collect();
        n.sort_by_key(|s| std::cmp::Reverse(s.len()));
        format!("`?(?:{})`?", n.join("|"))
    };
    let res = RE.get_or_init(|| {
        let n = names();
        [
            (
                format!(r"\s*\((?:fontes?|sources?|fuentes?):\s*{n}(?:\s*,\s*{n})*\s*\)"),
                "",
            ),
            (format!(r"\s*[;,]\s*{n}\s*\)"), ")"),
            (format!(r"\s*\(\s*{n}\s*\)"), ""),
            (format!(r"\s+[—–]\s+{n}\b"), ""),
            (
                format!(
                    r"(?i)\b(?:segundo|de acordo com|according to|según|de acuerdo con)\s+{n},?\s*"
                ),
                "",
            ),
        ]
        .into_iter()
        .map(|(p, r)| (regex::Regex::new(&p).expect("regex das ferramentas"), r))
        .collect()
    });
    let name = NAME.get_or_init(|| {
        let mut n: Vec<&str> = crate::mcp::tools::TOOLS.iter().map(|t| t.name).collect();
        n.sort_by_key(|s| std::cmp::Reverse(s.len()));
        regex::Regex::new(&format!(r"\b(?:{})\b", n.join("|"))).expect("regex dos nomes")
    });
    let mut found: Vec<String> = vec![];
    let mut out = text.to_string();
    for (re, rep) in res {
        for m in re.find_iter(&out) {
            for n in name.find_iter(m.as_str()) {
                if !found.iter().any(|f| f == n.as_str()) {
                    found.push(n.as_str().to_string());
                }
            }
        }
        out = re.replace_all(&out, *rep).into_owned();
    }
    (out, found)
}

/// Junta as fontes (ferramentas) achadas no texto às de `data.sources`.
fn with_sources(mut data: Value, found: &[String]) -> Value {
    if found.is_empty() {
        return data;
    }
    if let Some(m) = data.as_object_mut() {
        let list = m.entry("sources").or_insert_with(|| Value::Array(vec![]));
        if let Some(a) = list.as_array_mut() {
            for f in found {
                if !a.iter().any(|x| x == f) {
                    a.push(Value::String(f.clone()));
                }
            }
        }
    }
    data
}

/// Cria o alerta; `None` se a chave de deduplicação já existir.
pub fn create(conn: &Connection, a: &NewAlert<'_>) -> Result<Option<Id>> {
    let id = new_id();
    let now = now_utc();
    let n = conn.execute(
        "INSERT OR IGNORE INTO alert (id, kind, severity, title, body, status, source, dedupe_key, ref_table, ref_id, data, created_at, updated_at, title_key, args)
         VALUES (?1, ?2, ?3, ?4, ?5, 'new', ?6, ?7, ?8, ?9, ?10, ?11, ?11, ?12, ?13)",
        params![
            id,
            a.kind,
            a.severity.as_str(),
            a.title.render(Locale::PtBr),
            render_parts(&a.body, Locale::PtBr),
            a.source,
            a.dedupe_key,
            a.ref_table,
            a.ref_id,
            serde_json::to_string(&a.data)?,
            now,
            a.title.key,
            stored_args(Some(&a.title), &a.body)?,
        ],
    )?;
    Ok((n == 1).then_some(id))
}

/// O JSON da coluna `args` (V128).
#[derive(Debug, Default, Serialize, Deserialize)]
struct StoredArgs {
    /// Argumentos do título (a chave fica em `title_key`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    title: BTreeMap<String, Arg>,
    /// Partes do corpo, concatenadas.
    #[serde(default)]
    body: Vec<Part>,
}

fn stored_args(title: Option<&Msg>, body: &[Part]) -> Result<String> {
    Ok(serde_json::to_string(&StoredArgs {
        title: title.map(|t| t.args.clone()).unwrap_or_default(),
        body: body.to_vec(),
    })?)
}

/// O título e o corpo guardados pela chave: `None` no título (ou um corpo só com o texto
/// gravado) nas linhas de antes da V128.
fn stored_text(r: &rusqlite::Row<'_>) -> rusqlite::Result<(Option<Msg>, Option<Vec<Part>>)> {
    let key: Option<String> = r.get("title_key")?;
    let args: Option<StoredArgs> = r
        .get::<_, Option<String>>("args")?
        .and_then(|a| serde_json::from_str(&a).ok());
    let title = key.map(|key| Msg {
        key,
        args: args.as_ref().map(|a| a.title.clone()).unwrap_or_default(),
    });
    Ok((title, args.map(|a| a.body)))
}

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Alert> {
    let sev: String = r.get("severity")?;
    let st: String = r.get("status")?;
    let data: String = r.get("data")?;
    let l = i18n::current();
    let (title, body) = stored_text(r)?;
    // Alertas gravados antes da limpeza (ou com texto do motor) também saem sem os nomes das
    // ferramentas; eles vão para as fontes.
    let (body, found) = strip_tool_names(&match body {
        Some(b) => render_parts(&b, l),
        None => r.get::<_, String>("body")?,
    });
    let data = with_sources(serde_json::from_str(&data).unwrap_or(Value::Null), &found);
    Ok(Alert {
        id: r.get("id")?,
        kind: r.get("kind")?,
        severity: match sev.as_str() {
            "critical" => AlertSeverity::Critical,
            "warning" => AlertSeverity::Warning,
            _ => AlertSeverity::Info,
        },
        title: strip_tool_names(&match title {
            Some(t) => t.render(l),
            None => r.get::<_, String>("title")?,
        })
        .0,
        body,
        status: match st.as_str() {
            "seen" => AlertStatus::Seen,
            "resolved" => AlertStatus::Resolved,
            _ => AlertStatus::New,
        },
        source: source_label(l, &r.get::<_, String>("source")?),
        ref_table: r.get("ref_table")?,
        ref_id: r.get("ref_id")?,
        data,
        created_at: r.get("created_at")?,
        updated_at: r.get("updated_at")?,
        seen_at: r.get("seen_at")?,
        resolved_at: r.get("resolved_at")?,
        kind_label: String::new(),
        target: None,
    })
}

pub fn list(conn: &Connection, a: &AlertListArgs) -> Result<Vec<Alert>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM alert WHERE (?1 IS NULL AND status <> 'resolved' OR status = ?1)
           AND (?2 IS NULL OR kind = ?2)
         ORDER BY created_at DESC, id DESC LIMIT ?3",
    )?;
    Ok(stmt
        .query_map(
            params![
                a.status.map(AlertStatus::as_str),
                a.kind,
                a.limit.unwrap_or(50).clamp(1, 500)
            ],
            row,
        )?
        .map(|a| a.map(|a| enrich(conn, a)))
        .collect::<rusqlite::Result<_>>()?)
}

pub fn get(conn: &Connection, id: &str) -> Result<Alert> {
    conn.query_row("SELECT * FROM alert WHERE id = ?1", [id], row)
        .optional()?
        .map(|a| enrich(conn, a))
        .ok_or_else(|| Error::not_found(i18n::t("nucleo-ia-alerta")))
}

/// Alertas criados depois de `since`, ou reabertos para avisar de novo (`data.notify_at`), para
/// as notificações do sistema.
pub fn created_since(conn: &Connection, since: &str) -> Result<Vec<Alert>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM alert WHERE created_at > ?1 OR json_extract(data, '$.notify_at') > ?1
          ORDER BY created_at",
    )?;
    Ok(stmt
        .query_map([since], row)?
        .map(|a| a.map(|a| enrich(conn, a)))
        .collect::<rusqlite::Result<_>>()?)
}

pub fn counts(conn: &Connection) -> Result<AlertCounts> {
    Ok(conn.query_row(
        "SELECT COALESCE(SUM(status = 'new'), 0), COALESCE(SUM(status = 'seen'), 0) FROM alert",
        [],
        |r| {
            Ok(AlertCounts {
                new: r.get(0)?,
                seen: r.get(1)?,
            })
        },
    )?)
}

pub fn set_status(conn: &Connection, a: &AlertStatusArgs) -> Result<Vec<Alert>> {
    if a.ids.is_empty() {
        return Err(Error::invalid(i18n::t("nucleo-ia-alerta-selecione")));
    }
    let now = now_utc();
    let mut out = vec![];
    for id in &a.ids {
        let n = conn.execute(
            "UPDATE alert SET status = ?2, updated_at = ?3,
               seen_at = CASE WHEN ?2 IN ('seen', 'resolved') THEN COALESCE(seen_at, ?3) ELSE NULL END,
               resolved_at = CASE WHEN ?2 = 'resolved' THEN ?3 ELSE NULL END
             WHERE id = ?1",
            params![id, a.status.as_str(), now],
        )?;
        if n == 0 {
            return Err(Error::not_found(i18n::t_args(
                "nucleo-ia-alerta-id",
                &i18n::args(&[("id", id.to_string().into())]),
            )));
        }
        out.push(get(conn, id)?);
    }
    Ok(out)
}

/// O título (pela chave) e as partes do corpo de um alerta, para mudar só uma parte. Linha de
/// antes da V128: sem título pela chave e o corpo gravado como um texto só.
pub fn stored(conn: &Connection, id: &str) -> Result<(Option<Msg>, Vec<Part>)> {
    conn.query_row("SELECT * FROM alert WHERE id = ?1", [id], |r| {
        let (title, body) = stored_text(r)?;
        Ok((
            title,
            match body {
                Some(b) => b,
                None => vec![Part::Text(r.get("body")?)],
            },
        ))
    })
    .optional()?
    .ok_or_else(|| Error::not_found(i18n::t("nucleo-ia-alerta")))
}

/// Atualiza o texto e os números de um alerta no lugar (o mesmo episódio). Com `reopen`, volta
/// a ser novo (some do visto e do resolvido). Sem `title`, o título fica como está (linha de
/// antes da V128).
pub fn update_content(
    conn: &Connection,
    id: &str,
    severity: AlertSeverity,
    title: Option<&Msg>,
    body: &[Part],
    data: &Value,
    reopen: bool,
) -> Result<()> {
    // Sem título novo: o guardado (os argumentos dele vão na mesma coluna do corpo).
    let kept;
    let title = match title {
        Some(t) => Some(t),
        None => {
            kept = stored(conn, id)?.0;
            kept.as_ref()
        }
    };
    conn.execute(
        "UPDATE alert SET severity = ?2, title = COALESCE(?3, title), body = ?4, data = ?5,
                updated_at = ?6, title_key = COALESCE(?8, title_key), args = ?9,
                status = CASE WHEN ?7 THEN 'new' ELSE status END,
                seen_at = CASE WHEN ?7 THEN NULL ELSE seen_at END,
                resolved_at = CASE WHEN ?7 THEN NULL ELSE resolved_at END
          WHERE id = ?1",
        params![
            id,
            severity.as_str(),
            title.map(|t| t.render(Locale::PtBr)),
            render_parts(body, Locale::PtBr),
            serde_json::to_string(data)?,
            now_utc(),
            reopen,
            title.map(|t| t.key.as_str()),
            stored_args(title, body)?,
        ],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, ids: &[Id]) -> Result<i64> {
    let mut n = 0;
    for id in ids {
        n += conn.execute("DELETE FROM alert WHERE id = ?1", [id])? as i64;
    }
    Ok(n)
}

// ---------------------------------------------------------------------------
// Textos pela chave (V128)

/// Mensagem do catálogo (`locales/*/*.ftl`) com os argumentos, montada no idioma de quem lê.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Msg {
    pub key: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub args: BTreeMap<String, Arg>,
}

impl Msg {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            args: BTreeMap::new(),
        }
    }

    /// Com mais um argumento.
    pub fn arg(mut self, name: &str, v: impl Into<Arg>) -> Self {
        self.args.insert(name.to_string(), v.into());
        self
    }

    /// O texto no idioma `l`.
    pub fn render(&self, l: Locale) -> String {
        let a: i18n::Args = self
            .args
            .iter()
            .map(|(k, v)| (Cow::Owned(k.clone()), v.value(l)))
            .collect();
        i18n::tr(l, &self.key, Some(&a))
    }
}

/// Parte de um corpo: texto pronto (mostrado como está: o relatório do motor, um texto
/// antigo, pontuação e quebras de linha) ou mensagem do catálogo. O corpo é a concatenação.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Part {
    Text(String),
    Msg(Msg),
}

impl From<Msg> for Part {
    fn from(m: Msg) -> Self {
        Part::Msg(m)
    }
}

impl From<&str> for Part {
    fn from(s: &str) -> Self {
        Part::Text(s.to_string())
    }
}

/// As partes montadas no idioma `l`.
pub fn render_parts(parts: &[Part], l: Locale) -> String {
    parts
        .iter()
        .map(|p| match p {
            Part::Text(s) => s.clone(),
            Part::Msg(m) => m.render(l),
        })
        .collect()
}

/// Argumento de uma mensagem guardada. No JSON: texto (`"Itaú"`), número (`3`) ou um objeto
/// com o tipo (`{"money": [12345, "BRL"]}`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Arg {
    /// Como está: dado do usuário (conta, categoria, ativo), código, data ISO.
    Text(String),
    /// Número inteiro (contagens e seletores de plural do Fluent).
    Int(i64),
    /// Valor formatado no idioma de quem lê.
    Typed(Typed),
}

/// Valores formatados no idioma de quem lê.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Typed {
    /// Centavos e moeda ([`i18n::money`]).
    Money(i64, String),
    /// Data ISO por extenso ([`i18n::date`]: 14/10/2026, Oct 14, 2026).
    Date(String),
    /// Data ISO só com dia e mês ([`day_month`]: 14/10, Oct 14).
    DayMonth(String),
    /// Mês `AAAA-MM` ([`month_long`]: outubro de 2026, October 2026).
    Month(String),
    /// Fração como porcentagem, com as casas dadas ([`pct`]).
    Pct(f64, u8),
    /// Número com as casas dadas, no separador decimal do idioma.
    Dec(f64, u8),
    /// Número escrito com ponto (`"0.4523"`), no separador decimal do idioma.
    Decimal(String),
    /// Outra mensagem do catálogo.
    Msg(Msg),
    /// Outra mensagem do catálogo, em minúsculas.
    Lower(Msg),
    /// Vários argumentos separados por um texto (`"; "`).
    Join(Vec<Arg>, String),
}

impl Arg {
    pub fn money(cents: i64, currency: &str) -> Self {
        Arg::Typed(Typed::Money(cents, currency.to_string()))
    }
    pub fn date(iso: &str) -> Self {
        Arg::Typed(Typed::Date(iso.to_string()))
    }
    pub fn day_month(iso: &str) -> Self {
        Arg::Typed(Typed::DayMonth(iso.to_string()))
    }
    pub fn month(month: &str) -> Self {
        Arg::Typed(Typed::Month(month.to_string()))
    }
    pub fn pct(frac: f64, decimals: u8) -> Self {
        Arg::Typed(Typed::Pct(frac, decimals))
    }
    pub fn dec(v: f64, decimals: u8) -> Self {
        Arg::Typed(Typed::Dec(v, decimals))
    }
    pub fn decimal(s: &str) -> Self {
        Arg::Typed(Typed::Decimal(s.to_string()))
    }
    pub fn lower(m: Msg) -> Self {
        Arg::Typed(Typed::Lower(m))
    }
    pub fn join(items: Vec<Arg>, sep: &str) -> Self {
        Arg::Typed(Typed::Join(items, sep.to_string()))
    }

    /// O texto no idioma `l`.
    pub fn render(&self, l: Locale) -> String {
        match self {
            Arg::Text(s) => s.clone(),
            Arg::Int(n) => n.to_string(),
            Arg::Typed(t) => t.render(l),
        }
    }

    fn value(&self, l: Locale) -> fluent_bundle::FluentValue<'static> {
        match self {
            Arg::Int(n) => (*n).into(),
            other => other.render(l).into(),
        }
    }
}

impl Typed {
    fn render(&self, l: Locale) -> String {
        let iso = |d: &str| crate::time::parse_date(d).ok();
        match self {
            Typed::Money(c, cur) => i18n::money(l, *c, cur),
            Typed::Date(d) => iso(d).map_or_else(|| d.clone(), |x| i18n::date(l, x)),
            Typed::DayMonth(d) => iso(d).map_or_else(|| d.clone(), |x| day_month(l, x)),
            Typed::Month(m) => month_long(l, m),
            Typed::Pct(v, d) => pct(l, *v, *d),
            Typed::Dec(v, d) => decimal(l, &format!("{:.*}", *d as usize, v)),
            Typed::Decimal(s) => decimal(l, s),
            Typed::Msg(m) => m.render(l),
            Typed::Lower(m) => m.render(l).to_lowercase(),
            Typed::Join(items, sep) => items
                .iter()
                .map(|a| a.render(l))
                .collect::<Vec<_>>()
                .join(sep),
        }
    }
}

impl From<&str> for Arg {
    fn from(s: &str) -> Self {
        Arg::Text(s.to_string())
    }
}

impl From<String> for Arg {
    fn from(s: String) -> Self {
        Arg::Text(s)
    }
}

impl From<i64> for Arg {
    fn from(n: i64) -> Self {
        Arg::Int(n)
    }
}

impl From<Msg> for Arg {
    fn from(m: Msg) -> Self {
        Arg::Typed(Typed::Msg(m))
    }
}

// ---------------------------------------------------------------------------
// Formatos dos textos montados no núcleo (alertas, percepções, sugestões)

/// Número escrito com ponto (`"12.5"`) no separador decimal do idioma (12,5).
pub fn decimal(l: Locale, s: &str) -> String {
    let sep = i18n::number(l, 0).chars().nth(1).unwrap_or(',');
    s.replace('.', &sep.to_string())
}

/// Fração como porcentagem com `decimals` casas: 0,125 → 12,5% (pt-BR) ou 12.5% (en).
pub fn pct(l: Locale, frac: f64, decimals: u8) -> String {
    format!(
        "{}%",
        decimal(l, &format!("{:.*}", decimals as usize, frac * 100.0))
    )
}

/// Dia e mês: 14/10 (pt-BR, es) ou Oct 14 (en).
pub fn day_month(l: Locale, d: NaiveDate) -> String {
    match l {
        Locale::En => format!(
            "{} {}",
            i18n::month_names(l, true)[d.month0() as usize],
            d.day()
        ),
        _ => d.format("%d/%m").to_string(),
    }
}

fn month_with(l: Locale, month: &str, key: &str, short: bool) -> String {
    match crate::time::parse_month(month) {
        Ok((y, m)) => {
            let names = i18n::month_names(l, short);
            let name = names
                .get((m as usize).saturating_sub(1))
                .cloned()
                .unwrap_or_default();
            i18n::tr(
                l,
                key,
                Some(&i18n::args(&[
                    ("mes", name.into()),
                    ("ano", y.to_string().into()),
                ])),
            )
        }
        Err(_) => month.to_string(),
    }
}

/// Mês `AAAA-MM` por extenso: outubro de 2026, October 2026.
pub fn month_long(l: Locale, month: &str) -> String {
    month_with(l, month, "alertas-mes-ano", false)
}

/// Mês `AAAA-MM` abreviado: out/2026, Oct 2026.
pub fn month_short(l: Locale, month: &str) -> String {
    month_with(l, month, "alertas-mes-ano-curto", true)
}

/// A origem do alerta para mostrar (a gravada é um identificador em pt-BR, usado nas
/// consultas).
fn source_label(l: Locale, raw: &str) -> String {
    let key = match raw {
        "lembretes" => "alertas-origem-lembretes",
        "rotina: alertas" => "alertas-origem-rotina-alertas",
        "rotina: fechamento mensal" => "alertas-origem-fechamento",
        "score" => "alertas-origem-score",
        "sources" => "alertas-origem-fontes",
        _ => {
            return match raw
                .strip_prefix("rotina: fechamento mensal (")
                .and_then(|s| s.strip_suffix(')'))
            {
                Some(engine) => i18n::tr(
                    l,
                    "alertas-origem-fechamento-motor",
                    Some(&i18n::args(&[("motor", engine.to_string().into())])),
                ),
                None => raw.to_string(),
            };
        }
    };
    i18n::tr(l, key, None)
}

#[cfg(test)]
mod tidy_tests {
    #[test]
    fn glued_heading_goes_to_its_own_line() {
        assert_eq!(
            super::tidy_markdown("Vou somar os números.## Fechamento de outubro\n| a | b |"),
            "Vou somar os números.\n\n## Fechamento de outubro\n| a | b |"
        );
        // Título no lugar certo, cerquilha no meio do texto e tabela ficam como estão.
        for ok in [
            "Texto.\n\n### Receitas\n",
            "Linguagem C# é citada.",
            "| # | Item |\n|---|---|",
        ] {
            assert_eq!(super::tidy_markdown(ok), ok);
        }
    }
}
