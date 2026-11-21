//! Prévia e gravação das notas de corretagem.

use std::collections::HashMap;
use std::sync::LazyLock;

use base64::Engine as _;
use regex::Regex;
use rusqlite::{Connection, params};
use rust_decimal::Decimal;

use super::parse::{self, NoteLine, ParsedNote};
use super::*;
use crate::api::{Actor, Ctx};
use crate::assets::{self, AssetInput};
use crate::changes::Author;
use crate::error::{Error, Result};
use crate::i18n::{self, t, t_args};
use crate::import::b3::rules;
use crate::portfolio::records::{create_trade_with, to_minor, update_trade};
use crate::portfolio::{TradeInput, TradePatch};
use crate::time::fmt_date;

/// Texto de um arquivo: PDF (com senha opcional) ou texto já extraído.
fn file_text(f: &NoteFile, password: Option<&str>) -> std::result::Result<String, NoteFileError> {
    let err = |message: String, needs_password: bool| NoteFileError {
        filename: f.filename.clone(),
        message,
        needs_password,
    };
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(f.content_base64.trim())
        .map_err(|_| err(t("nucleo-importacao-erro-base64-curto"), false))?;
    if !bytes.starts_with(b"%PDF") {
        return String::from_utf8(bytes)
            .map_err(|_| err(t("nucleo-importacao-corretagem-erro-formato"), false));
    }
    crate::pdf::text(&bytes, password).map_err(|e| {
        let m = e;
        let lower = m.to_lowercase();
        if lower.contains("password") || lower.contains("decrypt") || lower.contains("encrypt") {
            err(
                t(if password.is_some() {
                    "nucleo-importacao-pdf-senha-errada"
                } else {
                    "nucleo-importacao-corretagem-pdf-senha"
                }),
                true,
            )
        } else {
            err(
                t_args(
                    "nucleo-importacao-pdf-erro",
                    &i18n::args(&[("erro", m.into())]),
                ),
                false,
            )
        }
    })
}

static TICKER_IN_SPEC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b([A-Z]{4}\d{1,2}F?)\b").expect("regex de ticker"));

/// Palavras da especificação que descrevem a classe da ação ou o segmento de listagem.
const CLASS_TOKENS: &[&str] = &[
    "ON", "PN", "PNA", "PNB", "PNC", "UNT", "UNIT", "N1", "N2", "NM", "MA", "MB", "EJ", "ED", "ER",
    "EB", "ES", "EX", "EDJ", "CI", "ERJ", "DRN", "DR1", "DR2", "DR3", "ATZ", "INT", "#", "@", "@#",
    "#@",
];

/// Sufixo do código de negociação pela classe da ação na especificação.
fn kind_suffix(spec: &str) -> Option<&'static str> {
    let toks: Vec<&str> = spec.split_whitespace().collect();
    let has = |t: &str| toks.contains(&t);
    if has("UNT") || has("UNIT") || has("CI") {
        Some("11")
    } else if has("DRN") {
        // BDR não patrocinado (o mais comum): final 34.
        Some("34")
    } else if has("PNA") {
        Some("5")
    } else if has("PNB") {
        Some("6")
    } else if has("PN") {
        Some("4")
    } else if has("ON") {
        Some("3")
    } else {
        None
    }
}

/// Nome da empresa na especificação, sem classe/segmento ("PETROBRAS PN N2" → "petrobras").
fn base_name(spec: &str) -> String {
    let kept: Vec<&str> = spec
        .split_whitespace()
        .filter(|t| !CLASS_TOKENS.contains(t))
        .filter(|t| *t != "FII")
        .collect();
    crate::transactions::normalize_description(&kept.join(" "))
}

/// Ajuste técnico: especificações já associadas a um ativo pelo usuário ou pelo agente
/// (`<nome>|<sufixo>` → id do ativo), para as próximas notas virem resolvidas.
pub const SPEC_MAP_SETTING: &str = "notes.spec_assets";

/// Chave estável da especificação: o nome sem classe, segmento e marcas de observação
/// ("FII RIZA TX CI ER @" e "FII RIZA TX CI" dão a mesma chave).
pub fn spec_key(spec: &str) -> String {
    format!("{}|{}", base_name(spec), kind_suffix(spec).unwrap_or(""))
}

fn remembered(conn: &Connection, spec: &str) -> Result<Option<String>> {
    let m: HashMap<String, String> =
        crate::settings::get_or(conn, SPEC_MAP_SETTING, HashMap::new())?;
    Ok(m.get(&spec_key(spec)).cloned())
}

/// Guarda as escolhas do usuário (ou do agente) para as próximas notas.
fn remember(conn: &Connection, choices: &HashMap<String, String>) -> Result<()> {
    if choices.is_empty() {
        return Ok(());
    }
    let mut m: HashMap<String, String> =
        crate::settings::get_or(conn, SPEC_MAP_SETTING, HashMap::new())?;
    for (spec, id) in choices {
        m.insert(spec_key(spec), id.clone());
    }
    crate::settings::set(conn, SPEC_MAP_SETTING, &m)
}

/// Escolha do usuário para uma especificação.
#[derive(Debug, Clone)]
enum Choice {
    Asset(String),
    New(NoteNewAsset),
}

type Choices = HashMap<String, Choice>;

fn chosen<'a>(choices: &'a Choices, spec: &str) -> Option<&'a Choice> {
    choices.get(spec).or_else(|| {
        let k = spec_key(spec);
        choices
            .iter()
            .find(|(s, _)| spec_key(s) == k)
            .map(|(_, c)| c)
    })
}

/// Ajuste técnico: nomes de pregão da B3 (código → (NOMRES, ESPECI, BDI)), atualizado a
/// cada cotação diária coletada pela análise de ativos.
pub const TRADING_NAMES_SETTING: &str = "b3.trading_names";

/// Marcas de observação e de direitos que a nota acrescenta e o COTAHIST nem sempre traz
/// ("@" negócio direto, "#" ajuste, "ER"/"EJ" ex-rendimento/ex-juros…).
const MARK_TOKENS: &[&str] = &[
    "@", "#", "@#", "#@", "ER", "EJ", "ED", "EX", "EB", "ES", "EDJ", "ERJ", "EDR", "ATZ", "INT",
];

/// Guarda os nomes de pregão das cotações diárias da B3.
pub fn store_trading_names(conn: &Connection, quotes: &[crate::research::b3::Quote]) -> Result<()> {
    if quotes.is_empty() {
        return Ok(());
    }
    let mut names: HashMap<String, (String, String, String)> =
        crate::settings::get_or(conn, TRADING_NAMES_SETTING, HashMap::new())?;
    for q in quotes.iter().filter(|q| !q.short_name.is_empty()) {
        names.insert(
            q.ticker.clone(),
            (q.short_name.clone(), q.spec.clone(), q.bdi.clone()),
        );
    }
    crate::settings::set(conn, TRADING_NAMES_SETTING, &names)
}

fn strip_marks(s: &str) -> String {
    s.split_whitespace()
        .filter(|t| !MARK_TOKENS.contains(t))
        .collect::<Vec<_>>()
        .join(" ")
}

fn class_of_bdi(bdi: &str, ticker: &str) -> crate::assets::AssetClass {
    use crate::assets::AssetClass;
    match bdi {
        "12" => AssetClass::Fii,
        "14" => AssetClass::EtfBr,
        "34" | "35" | "36" => AssetClass::Bdr,
        _ if ticker.ends_with("34") || ticker.ends_with("35") || ticker.ends_with("39") => {
            AssetClass::Bdr
        }
        _ => AssetClass::StockBr,
    }
}

/// Código de negociação pelo nome de pregão: "FII GGRCOVEP CI ER" casa com NOMRES
/// "FII GGRCOVEP" + ESPECI "CI". Sem casamento exato, aceita o nome igual com a classe
/// compatível com o final do código, quando só um código combina.
pub(crate) fn by_trading_name(
    conn: &Connection,
    spec: &str,
) -> Result<Option<(String, crate::assets::AssetClass)>> {
    let names: HashMap<String, (String, String, String)> =
        crate::settings::get_or(conn, TRADING_NAMES_SETTING, HashMap::new())?;
    if names.is_empty() {
        return Ok(None);
    }
    let want = strip_marks(spec);
    let mut exact: Vec<(&String, &String)> = names
        .iter()
        .filter(|(_, (n, e, _))| strip_marks(&format!("{n} {e}")) == want)
        .map(|(t, (_, _, b))| (t, b))
        .collect();
    if exact.is_empty() {
        let suffix = kind_suffix(spec);
        exact = names
            .iter()
            .filter(|(t, (n, _, _))| {
                let n = &strip_marks(n);
                !n.is_empty()
                    && (want == *n || want.starts_with(&format!("{n} ")))
                    && suffix.is_none_or(|s| t.get(4..) == Some(s))
            })
            .map(|(t, (_, _, b))| (t, b))
            .collect();
    }
    if exact.is_empty() {
        // Nome de pregão trocado depois da nota ("FII GGRCOVEP" virou "FII GGR ZAG"): a
        // raiz do código costuma ser o começo de uma palavra do nome antigo, com a mesma
        // especificação.
        let words: Vec<&str> = want
            .split_whitespace()
            .filter(|w| w.len() >= 4 && *w != "FII")
            .collect();
        let class = strip_marks(
            &spec
                .split_whitespace()
                .filter(|t| CLASS_TOKENS.contains(t))
                .collect::<Vec<_>>()
                .join(" "),
        );
        exact = names
            .iter()
            .filter(|(t, (_, e, _))| {
                t.len() > 4
                    && strip_marks(e) == class
                    && t.get(..4)
                        .is_some_and(|p| words.iter().any(|w| w.starts_with(p)))
            })
            .map(|(t, (_, _, b))| (t, b))
            .collect();
    }
    // Lote padrão e fracionário dividem o nome: o fracionário já foi filtrado pelo BDI.
    exact.sort();
    exact.dedup();
    Ok(match exact.as_slice() {
        [(t, b)] => Some(((*t).clone(), class_of_bdi(b, t))),
        _ => None,
    })
}

#[derive(Debug, Clone)]
enum Resolution {
    Asset { id: String, ticker: Option<String> },
    Create(NoteNewAsset),
    Unresolved(String),
}

fn resolve(conn: &Connection, spec: &str, market: &str, choices: &Choices) -> Result<Resolution> {
    match chosen(choices, spec) {
        Some(Choice::Asset(id)) => {
            let a = assets::get(conn, id)?;
            return Ok(Resolution::Asset {
                id: a.id,
                ticker: a.ticker,
            });
        }
        Some(Choice::New(a)) => return Ok(Resolution::Create(a.clone())),
        None => {}
    }
    // 0) Associação já feita numa importação anterior (se o ativo ainda existe).
    if let Some(id) = remembered(conn, spec)?
        && let Ok(a) = assets::get(conn, &id)
    {
        return Ok(Resolution::Asset {
            id: a.id,
            ticker: a.ticker,
        });
    }
    if market.starts_with("OPCAO") || market.starts_with("EXERC") || market == "TERMO" {
        return Ok(Resolution::Unresolved(t(
            "nucleo-importacao-corretagem-ativo-opcao",
        )));
    }
    // 1) Código de negociação escrito na especificação (FIIs costumam trazer).
    for c in TICKER_IN_SPEC.captures_iter(spec) {
        let Some(t) = rules::normalize_ticker(&c[1]) else {
            continue;
        };
        if let Some(a) = assets::find_by_ticker(conn, &t)? {
            return Ok(Resolution::Asset {
                id: a.id,
                ticker: a.ticker,
            });
        }
        let g = rules::infer_class(Some(&t), spec, None, true);
        let class = if spec.split_whitespace().any(|w| w == "FII" || w == "CI") {
            crate::assets::AssetClass::Fii
        } else {
            g.class
        };
        return Ok(Resolution::Create(NoteNewAsset {
            ticker: t,
            name: spec.to_string(),
            class,
        }));
    }
    // 1b) Nome de pregão + especificação da B3 (COTAHIST): é exatamente o que a nota traz.
    if let Some((ticker, class)) = by_trading_name(conn, spec)? {
        if let Some(a) = assets::find_by_ticker(conn, &ticker)? {
            return Ok(Resolution::Asset {
                id: a.id,
                ticker: a.ticker,
            });
        }
        return Ok(Resolution::Create(NoteNewAsset {
            ticker,
            name: spec.to_string(),
            class,
        }));
    }
    let name = base_name(spec);
    if name.is_empty() {
        return Ok(Resolution::Unresolved(t(
            "nucleo-importacao-corretagem-ativo-sem-nome",
        )));
    }
    let suffix = kind_suffix(spec);
    let suffix_ok = |t: &str| match suffix {
        Some(s) => t.len() > 4 && t.get(4..) == Some(s),
        None => true,
    };
    // 2) Ativos cadastrados com o mesmo nome (e a classe da ação compatível com o código).
    let mut stmt = conn.prepare(
        "SELECT id, ticker, name FROM asset WHERE ticker IS NOT NULL AND class IN ('stock_br', 'fii', 'etf_br', 'bdr')",
    )?;
    let rows: Vec<(String, String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let hits: Vec<&(String, String, String)> = rows
        .iter()
        .filter(|(_, t, n)| {
            let n = crate::transactions::normalize_description(n);
            (n.starts_with(&name) || name.starts_with(&n)) && suffix_ok(t)
        })
        .collect();
    if let [(id, t, _)] = hits.as_slice() {
        return Ok(Resolution::Asset {
            id: id.clone(),
            ticker: Some(t.clone()),
        });
    }
    // 2b) Raiz do código na especificação ("FII HGLG PAX CI" → HGLG11), com a classe.
    let roots: Vec<String> = spec
        .split_whitespace()
        .filter(|w| w.len() == 4 && w.chars().all(|c| c.is_ascii_uppercase()))
        .map(str::to_string)
        .collect();
    let by_root: Vec<&(String, String, String)> = rows
        .iter()
        .filter(|(_, t, _)| {
            t.len() > 4 && roots.iter().any(|r| t.starts_with(r.as_str())) && suffix_ok(t)
        })
        .collect();
    if let [(id, t, _)] = by_root.as_slice() {
        return Ok(Resolution::Asset {
            id: id.clone(),
            ticker: Some(t.clone()),
        });
    }
    // 2c) Primeira palavra do nome igual à do ativo ("TESLA INC DRN" → "Tesla BDR", final
    // 34), quando só um combina.
    if let Some(first) = name.split_whitespace().next().filter(|w| w.len() >= 4) {
        let by_word: Vec<&(String, String, String)> = rows
            .iter()
            .filter(|(_, t, n)| {
                crate::transactions::normalize_description(n)
                    .split_whitespace()
                    .next()
                    == Some(first)
                    && suffix_ok(t)
            })
            .collect();
        if let [(id, t, _)] = by_word.as_slice() {
            return Ok(Resolution::Asset {
                id: id.clone(),
                ticker: Some(t.clone()),
            });
        }
    }
    // 3) Cadastro da CVM (análise de ativos), quando já coletado.
    let mut stmt = conn.prepare(
        "SELECT s.ticker, s.class, i.name FROM research_security s JOIN research_issuer i ON i.id = s.issuer_id WHERE s.active = 1",
    )?;
    let rows: Vec<(String, String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let hits: Vec<&(String, String, String)> = rows
        .iter()
        .filter(|(t, _, n)| {
            crate::transactions::normalize_description(n).starts_with(&name) && suffix_ok(t)
        })
        .collect();
    if let [(t, class, _)] = hits.as_slice() {
        if let Some(a) = assets::find_by_ticker(conn, t)? {
            return Ok(Resolution::Asset {
                id: a.id,
                ticker: a.ticker,
            });
        }
        let class =
            crate::assets::AssetClass::parse(class).unwrap_or(crate::assets::AssetClass::StockBr);
        return Ok(Resolution::Create(NoteNewAsset {
            ticker: t.clone(),
            name: spec.to_string(),
            class,
        }));
    }
    Ok(Resolution::Unresolved(t(if hits.len() > 1 {
        "nucleo-importacao-corretagem-ativo-varios"
    } else {
        "nucleo-importacao-corretagem-ativo-nenhum"
    })))
}

/// Soma as execuções da mesma ordem: mesmo título (sem as marcas de observação), lado,
/// mercado e day trade. A quantidade e o valor somam; o preço é o médio ponderado.
fn group_lines(lines: &[NoteLine]) -> Vec<(NoteLine, u32)> {
    let mut out: Vec<(NoteLine, u32)> = vec![];
    for l in lines {
        let dt = l.obs.as_deref() == Some("D");
        let key = spec_key(&l.spec);
        if let Some((g, n)) = out.iter_mut().find(|(g, _)| {
            g.side == l.side
                && g.market == l.market
                && (g.obs.as_deref() == Some("D")) == dt
                && spec_key(&g.spec) == key
        }) {
            g.quantity += l.quantity;
            g.value += l.value;
            *n += 1;
        } else {
            out.push((l.clone(), 1));
        }
    }
    for (g, n) in &mut out {
        if *n > 1 && !g.quantity.is_zero() {
            g.price = (g.value / g.quantity).round_dp(6).normalize();
            // A marca de uma execução ("@", "ER"…) não descreve a ordem toda.
            g.spec = strip_marks(&g.spec);
        }
    }
    out
}

/// Reparte `total` (centavos) proporcionalmente aos pesos, fechando a soma exata.
fn allocate(total: i64, weights: &[i64]) -> Vec<i64> {
    let sum: i64 = weights.iter().sum();
    if total == 0 || sum <= 0 {
        return vec![0; weights.len()];
    }
    let mut out: Vec<i64> = weights
        .iter()
        .map(|w| (total as i128 * *w as i128 / sum as i128) as i64)
        .collect();
    let rest = total - out.iter().sum::<i64>();
    // Sobra do arredondamento para as maiores operações.
    let mut order: Vec<usize> = (0..weights.len()).collect();
    order.sort_by_key(|i| std::cmp::Reverse(weights[*i]));
    for i in order.iter().cycle().take(rest.unsigned_abs() as usize) {
        out[*i] += rest.signum();
    }
    out
}

fn cents(v: Decimal) -> i64 {
    to_minor(v, "BRL")
}

struct Planned {
    preview: NotePreview,
    /// Por índice: entrada pronta para gravar, ativo a criar e trade existente a completar.
    items: Vec<PlannedItem>,
}

struct PlannedItem {
    index: u32,
    status: NoteTradeStatus,
    input: TradeInput,
    create_asset: Option<NoteNewAsset>,
    /// Operações já gravadas (mais de uma quando as execuções agrupadas foram gravadas
    /// separadas antes).
    existing: Vec<String>,
}

fn plan(
    conn: &Connection,
    files: &[NoteFile],
    account_id: &str,
    password: Option<&str>,
    choices: &Choices,
    group: bool,
) -> Result<Planned> {
    let (currency, _) = crate::accounts::currency_and_kind(conn, account_id)?;
    if currency != "BRL" {
        return Err(Error::invalid(t("nucleo-importacao-corretagem-erro-brl")));
    }
    let mut preview = NotePreview {
        notes: vec![],
        trades: vec![],
        unresolved_specs: vec![],
        errors: vec![],
    };
    let mut items = vec![];
    let mut index = 0u32;
    for f in files {
        let text = match file_text(f, password) {
            Ok(t) => t,
            Err(e) => {
                preview.errors.push(e);
                continue;
            }
        };
        let notes = parse::parse_text(&text);
        if notes.is_empty() {
            // O motivo pelos layouts (docs/layouts.md): nota de layout desconhecido (talvez
            // uma versão nova) ou arquivo de outro assunto.
            let message = crate::layouts::require(
                &crate::layouts::Doc {
                    format: crate::layouts::LayoutFormat::Pdf,
                    text: &text,
                    headers: &[],
                },
                crate::layouts::LayoutSubject::BrokerageNote,
            )
            .err()
            .map(|e| e.message())
            .unwrap_or_else(|| t("nucleo-importacao-corretagem-erro-nenhuma"));
            preview.errors.push(NoteFileError {
                filename: f.filename.clone(),
                message,
                needs_password: false,
            });
            continue;
        }
        for n in notes {
            plan_note(
                conn,
                f,
                &n,
                account_id,
                choices,
                group,
                &mut index,
                &mut preview,
                &mut items,
            )?;
        }
    }
    preview.unresolved_specs.sort();
    preview.unresolved_specs.dedup();
    check_positions(conn, account_id, &mut preview)?;
    Ok(Planned { preview, items })
}

/// Aponta vendas maiores que a posição na data (contando as compras já gravadas e as
/// desta importação, em ordem de data). Não impede gravar: a compra pode ter sido em outra
/// corretora ou num período ainda não importado.
fn check_positions(conn: &Connection, account_id: &str, preview: &mut NotePreview) -> Result<()> {
    let mut order: Vec<usize> = (0..preview.trades.len()).collect();
    order.sort_by(|&a, &b| {
        let (x, y) = (&preview.trades[a], &preview.trades[b]);
        x.date
            .cmp(&y.date)
            .then((x.side == TradeSide::Sell).cmp(&(y.side == TradeSide::Sell)))
    });
    // Saldo das operações novas desta importação, por ativo (id ou código a cadastrar).
    let mut added: HashMap<String, Decimal> = HashMap::new();
    for i in order {
        let t = &preview.trades[i];
        if t.status != NoteTradeStatus::New {
            continue;
        }
        let key = match (&t.asset_id, &t.create_asset) {
            (Some(id), _) => id.clone(),
            (None, Some(a)) => format!("novo:{}", a.ticker),
            _ => continue,
        };
        let qty: Decimal = t.quantity.parse().unwrap_or_default();
        let delta = added.entry(key.clone()).or_default();
        if t.side == TradeSide::Buy {
            *delta += qty;
            continue;
        }
        let stored = match &t.asset_id {
            Some(id) => {
                crate::portfolio::engine::quantity_held(conn, id, Some(account_id), &t.date)?
            }
            None => Decimal::ZERO,
        };
        let held = stored + *delta;
        *delta -= qty;
        if qty > held {
            let fmt = |d: Decimal| crate::portfolio::records::fmt_qty(d.max(Decimal::ZERO));
            // Duas partes unidas por "; ": a interface mostra só a primeira no resumo.
            preview.trades[i].warning = Some(format!(
                "{}; {}",
                t_args(
                    "nucleo-importacao-corretagem-venda-maior",
                    &i18n::args(&[("venda", fmt(qty).into()), ("posicao", fmt(held).into())]),
                ),
                i18n::t("nucleo-importacao-corretagem-venda-grava")
            ));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn plan_note(
    conn: &Connection,
    f: &NoteFile,
    n: &ParsedNote,
    account_id: &str,
    choices: &Choices,
    group: bool,
    index: &mut u32,
    preview: &mut NotePreview,
    items: &mut Vec<PlannedItem>,
) -> Result<()> {
    let costs = NoteCosts {
        settlement: cents(n.fees.settlement),
        registration: cents(n.fees.registration),
        emoluments: cents(n.fees.emoluments),
        brokerage: cents(n.fees.brokerage),
        iss: cents(n.fees.iss),
        other: cents(n.fees.other),
        irrf: cents(n.fees.irrf),
        total: cents(n.fees.costs()),
    };
    let mut warnings = n.warnings.clone();
    let Some(date) = n.date else {
        warnings.push(t("nucleo-importacao-corretagem-sem-data"));
        preview.notes.push(summary(f, n, costs, 0, warnings));
        return Ok(());
    };
    let date_s = fmt_date(date);
    let lines = if group {
        group_lines(&n.lines)
    } else {
        n.lines.iter().map(|l| (l.clone(), 1)).collect()
    };
    let gross: Vec<i64> = lines.iter().map(|(l, _)| cents(l.value)).collect();
    let fees = allocate(costs.total, &gross);
    let sell_w: Vec<i64> = lines
        .iter()
        .zip(&gross)
        .map(|((l, _), g)| if l.side == TradeSide::Sell { *g } else { 0 })
        .collect();
    let irrf = allocate(costs.irrf, &sell_w);
    // Compra e venda do mesmo título na mesma nota: day trade.
    let both = |l: &NoteLine| n.lines.iter().any(|o| o.spec == l.spec && o.side != l.side);
    let computed_net = lines
        .iter()
        .zip(&gross)
        .map(|((l, _), g)| if l.side == TradeSide::Sell { *g } else { -g })
        .sum::<i64>()
        - costs.total
        - costs.irrf;
    for (k, (l, executions)) in lines.iter().enumerate() {
        let user_choice = chosen(choices, &l.spec).is_some();
        let res = resolve(conn, &l.spec, &l.market, choices)?;
        let (asset_id, ticker, create, unresolved) = match res {
            Resolution::Asset { id, ticker } => (Some(id), ticker, None, None),
            Resolution::Create(a) => (None, Some(a.ticker.clone()), Some(a), None),
            Resolution::Unresolved(why) => (None, None, None, Some(why)),
        };
        let day_trade = l.obs.as_deref() == Some("D") || both(l);
        let executions = *executions;
        let mut status = NoteTradeStatus::New;
        let mut existing: Vec<String> = vec![];
        let mut note = unresolved.clone();
        if unresolved.is_some() {
            status = NoteTradeStatus::Unresolved;
            preview.unresolved_specs.push(l.spec.clone());
        } else if let Some(aid) = &asset_id
            && let Some(ExistingTrade {
                id,
                fees: fees_now,
                taxes: taxes_now,
                note_number: number,
            }) = same_trade(conn, account_id, aid, &date_s, l, n.number.as_deref())?
        {
            existing = id;
            if number.is_some() && number == n.number {
                status = NoteTradeStatus::Duplicate;
                note = Some(t("nucleo-importacao-corretagem-ja-importada"));
            } else if fees_now == 0 && taxes_now == 0 {
                status = NoteTradeStatus::FillCosts;
                note = Some(t("nucleo-importacao-corretagem-sem-custos"));
            } else {
                status = NoteTradeStatus::Duplicate;
                note = Some(t("nucleo-importacao-corretagem-com-custos"));
            }
        }
        let i = *index;
        *index += 1;
        let input = TradeInput {
            asset_id: asset_id.clone().unwrap_or_default(),
            account_id: account_id.to_string(),
            date: date_s.clone(),
            side: l.side,
            quantity: l.quantity.normalize().to_string(),
            price: l.price.normalize().to_string(),
            fees: Some(fees[k]),
            taxes: Some(irrf[k]),
            fx_rate: None,
            day_trade: Some(day_trade),
            note_number: n.number.clone(),
            notes: None,
            cash_movement: None,
            external_id: None,
        };
        preview.trades.push(NoteTrade {
            index: i,
            filename: f.filename.clone(),
            note_number: n.number.clone(),
            date: date_s.clone(),
            side: l.side,
            market: l.market.clone(),
            spec: l.spec.clone(),
            ticker,
            asset_id,
            create_asset: create.clone(),
            quantity: input.quantity.clone(),
            price: input.price.clone(),
            gross: gross[k],
            fees: fees[k],
            taxes: irrf[k],
            day_trade,
            executions,
            chosen: user_choice,
            status,
            warning: None,
            existing_trade_id: existing.first().cloned(),
            note,
        });
        items.push(PlannedItem {
            index: i,
            status,
            input,
            create_asset: create,
            existing,
        });
    }
    if let Some(net) = n.net.map(cents)
        && (net - computed_net).abs() > 1
    {
        warnings.push(t_args(
            "nucleo-importacao-corretagem-liquido-diferente",
            &i18n::args(&[
                (
                    "nota",
                    crate::transactions::util::money_text(net, "BRL").into(),
                ),
                (
                    "calculado",
                    crate::transactions::util::money_text(computed_net, "BRL").into(),
                ),
            ]),
        ));
    }
    preview
        .notes
        .push(summary(f, n, costs, computed_net, warnings));
    Ok(())
}

fn summary(
    f: &NoteFile,
    n: &ParsedNote,
    costs: NoteCosts,
    computed_net: i64,
    warnings: Vec<String>,
) -> NoteSummary {
    NoteSummary {
        filename: f.filename.clone(),
        number: n.number.clone(),
        date: n.date.map(fmt_date),
        trades: n.lines.len() as u32,
        costs,
        net: n.net.map(cents),
        computed_net,
        warnings,
    }
}

struct ExistingTrade {
    id: Vec<String>,
    fees: i64,
    taxes: i64,
    note_number: Option<String>,
}

/// Operação já gravada igual (conta, ativo, data, lado, quantidade e preço).
/// id, quantidade, preço, taxas, impostos e número da nota de uma operação gravada.
type TradeRow = (String, String, String, i64, i64, Option<String>);

fn same_trade(
    conn: &Connection,
    account_id: &str,
    asset_id: &str,
    date: &str,
    l: &NoteLine,
    note_number: Option<&str>,
) -> Result<Option<ExistingTrade>> {
    let side = if l.side == TradeSide::Buy {
        "buy"
    } else {
        "sell"
    };
    let mut stmt = conn.prepare(
        "SELECT id, quantity, price, fees, taxes, note_number FROM trade
         WHERE account_id = ?1 AND asset_id = ?2 AND date = ?3 AND side = ?4 ORDER BY created_at",
    )?;
    #[allow(clippy::type_complexity)]
    let rows: Vec<(String, String, String, i64, i64, Option<String>)> = stmt
        .query_map(params![account_id, asset_id, date, side], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    for (id, q, p, fees, taxes, number) in &rows {
        let (Ok(q), Ok(p)) = (q.parse::<Decimal>(), p.parse::<Decimal>()) else {
            continue;
        };
        if q == l.quantity && (p - l.price).abs() < Decimal::new(1, 2) {
            return Ok(Some(ExistingTrade {
                id: vec![id.clone()],
                fees: *fees,
                taxes: *taxes,
                note_number: number.clone(),
            }));
        }
    }
    // Execuções gravadas separadas (importação anterior sem agrupar, ou o relatório da
    // B3): a soma das do mesmo dia (dessa nota, quando já têm número) fecha a quantidade e
    // o valor.
    let same: Vec<&TradeRow> = rows
        .iter()
        .filter(|r| r.5.is_none() || r.5.as_deref() == note_number)
        .collect();
    if same.len() > 1 {
        let (mut qty, mut value) = (Decimal::ZERO, Decimal::ZERO);
        for (_, q, p, ..) in &same {
            let (Ok(q), Ok(p)) = (q.parse::<Decimal>(), p.parse::<Decimal>()) else {
                return Ok(None);
            };
            qty += q;
            value += q * p;
        }
        if qty == l.quantity && (value - l.value).abs() < Decimal::new(5, 2) {
            let number = same.iter().find_map(|r| r.5.clone());
            return Ok(Some(ExistingTrade {
                id: same.iter().map(|r| r.0.clone()).collect(),
                fees: same.iter().map(|r| r.3).sum(),
                taxes: same.iter().map(|r| r.4).sum(),
                note_number: number,
            }));
        }
    }
    Ok(None)
}

fn choices_map(conn: &Connection, c: Option<&Vec<NoteAssetChoice>>) -> Result<Choices> {
    let mut m = HashMap::new();
    for c in c.into_iter().flatten() {
        let choice = if let Some(id) = c.asset_id.as_deref().filter(|s| !s.is_empty()) {
            assets::get(conn, id)?;
            Choice::Asset(id.to_string())
        } else if let Some(raw) = c.ticker.as_deref().filter(|s| !s.trim().is_empty()) {
            let Some(t) = rules::normalize_ticker(raw) else {
                return Err(Error::invalid(t_args(
                    "nucleo-importacao-corretagem-erro-codigo",
                    &i18n::args(&[("codigo", raw.to_string().into())]),
                )));
            };
            if let Some(a) = assets::find_by_ticker(conn, &t)? {
                Choice::Asset(a.id)
            } else {
                let class = match c.class {
                    Some(k) => k,
                    None => match by_trading_name_of(conn, &t)? {
                        Some(k) => k,
                        None => rules::infer_class(Some(&t), &c.spec, None, true).class,
                    },
                };
                Choice::New(NoteNewAsset {
                    ticker: t,
                    name: c.spec.clone(),
                    class,
                })
            }
        } else {
            return Err(Error::invalid(t_args(
                "nucleo-importacao-corretagem-erro-escolha",
                &i18n::args(&[("especificacao", c.spec.clone().into())]),
            )));
        };
        m.insert(c.spec.clone(), choice);
    }
    Ok(m)
}

/// Classe pelo BDI do código no COTAHIST, quando conhecido.
fn by_trading_name_of(
    conn: &Connection,
    ticker: &str,
) -> Result<Option<crate::assets::AssetClass>> {
    let names: HashMap<String, (String, String, String)> =
        crate::settings::get_or(conn, TRADING_NAMES_SETTING, HashMap::new())?;
    Ok(names.get(ticker).map(|(_, _, b)| class_of_bdi(b, ticker)))
}

pub fn preview(ctx: &mut Ctx<'_>, args: &NotePreviewArgs) -> Result<NotePreview> {
    if args.files.is_empty() {
        return Err(Error::invalid(t(
            "nucleo-importacao-corretagem-erro-sem-arquivo",
        )));
    }
    Ok(plan(
        ctx.conn(),
        &args.files,
        &args.account_id,
        args.password.as_deref(),
        &choices_map(ctx.conn(), args.assets.as_ref())?,
        args.group.unwrap_or(true),
    )?
    .preview)
}

fn trade_value_cents(conn: &Connection, id: &str) -> Result<i64> {
    let (q, p): (String, String) = conn.query_row(
        "SELECT quantity, price FROM trade WHERE id = ?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let v = q.parse::<Decimal>().unwrap_or_default() * p.parse::<Decimal>().unwrap_or_default();
    Ok(cents(v).max(1))
}

pub fn commit(ctx: &mut Ctx<'_>, args: &NoteCommitArgs) -> Result<NoteCommitResult> {
    if args.files.is_empty() {
        return Err(Error::invalid(t(
            "nucleo-importacao-corretagem-erro-sem-arquivo",
        )));
    }
    let choices = choices_map(ctx.conn(), args.assets.as_ref())?;
    let picked: HashMap<String, String> = choices
        .iter()
        .filter_map(|(s, c)| match c {
            Choice::Asset(id) => Some((s.clone(), id.clone())),
            Choice::New(_) => None,
        })
        .collect();
    remember(ctx.conn(), &picked)?;
    let planned = plan(
        ctx.conn(),
        &args.files,
        &args.account_id,
        args.password.as_deref(),
        &choices,
        args.group.unwrap_or(true),
    )?;
    if let Some(e) = planned.preview.errors.first() {
        return Err(Error::invalid(format!("{}: {}", e.filename, e.message)));
    }
    let skip: Vec<u32> = args.skip.clone().unwrap_or_default();
    let todo: Vec<&PlannedItem> = planned
        .items
        .iter()
        .filter(|i| !skip.contains(&i.index))
        .filter(|i| matches!(i.status, NoteTradeStatus::New | NoteTradeStatus::FillCosts))
        .collect();
    let skipped = planned.items.len() as u32 - todo.len() as u32;
    // Em ordem de data, compras antes das vendas do mesmo dia.
    let mut todo = todo;
    todo.sort_by(|a, b| {
        a.input
            .date
            .cmp(&b.input.date)
            .then((a.input.side == TradeSide::Sell).cmp(&(b.input.side == TradeSide::Sell)))
    });
    if todo.is_empty() {
        return Ok(NoteCommitResult {
            created: 0,
            updated: 0,
            assets_created: 0,
            skipped,
            change_set_id: None,
        });
    }
    let files: Vec<&str> = args.files.iter().map(|f| f.filename.as_str()).collect();
    let files = files.join(", ");
    let detail = t_args(
        "nucleo-importacao-corretagem-autor",
        &i18n::args(&[("arquivos", files.clone().into())]),
    );
    let author = match ctx.actor {
        Actor::Mcp { client, .. } => Author::import(format!("{detail} — {client}")),
        _ => Author::import(detail),
    };
    let reason = t_args(
        "nucleo-importacao-corretagem-historico",
        &i18n::args(&[("arquivos", files.clone().into())]),
    );
    let cash = args.cash_movement.unwrap_or(false);
    let rec = ctx.write_as(&author, &reason, |tx| {
        let (mut created, mut updated, mut assets_created) = (0u32, 0u32, 0u32);
        let mut new_assets: HashMap<String, String> = HashMap::new();
        for it in &todo {
            match it.status {
                NoteTradeStatus::FillCosts => {
                    // Execuções gravadas separadas: os custos da ordem são repartidos pelo
                    // valor de cada uma.
                    let weights: Vec<i64> = it
                        .existing
                        .iter()
                        .map(|id| trade_value_cents(tx, id))
                        .collect::<Result<_>>()?;
                    let fees = allocate(it.input.fees.unwrap_or(0), &weights);
                    let taxes = allocate(it.input.taxes.unwrap_or(0), &weights);
                    for (k, id) in it.existing.iter().enumerate() {
                        update_trade(
                            tx,
                            &TradePatch {
                                id: id.clone(),
                                date: None,
                                side: None,
                                quantity: None,
                                price: None,
                                fees: Some(fees[k]),
                                taxes: Some(taxes[k]),
                                fx_rate: None,
                                day_trade: it.input.day_trade,
                                note_number: Some(it.input.note_number.clone()),
                                notes: None,
                                cash_movement: None,
                            },
                        )?;
                        updated += 1;
                    }
                }
                _ => {
                    let mut input = it.input.clone();
                    if let Some(a) = &it.create_asset {
                        let id = match new_assets.get(&a.ticker) {
                            Some(id) => id.clone(),
                            None => {
                                let id = assets::create(
                                    tx,
                                    &AssetInput {
                                        ticker: Some(a.ticker.clone()),
                                        name: a.name.clone(),
                                        official_name: None,
                                        account_id: None,
                                        class: a.class,
                                        market: None,
                                        currency: None,
                                        country: None,
                                        isin: None,
                                        cnpj: None,
                                        treasury_code: None,
                                        sector: None,
                                        segment: None,
                                        price_provider: None,
                                        provider_symbol: None,
                                        notes: Some(t(
                                            "nucleo-importacao-corretagem-ativo-cadastrado",
                                        )),
                                        fixed_income: None,
                                        pension: None,
                                    },
                                )?;
                                assets_created += 1;
                                new_assets.insert(a.ticker.clone(), id.clone());
                                id
                            }
                        };
                        input.asset_id = id;
                    }
                    input.cash_movement = Some(cash);
                    // Venda maior que a posição grava com aviso (a prévia já apontou).
                    create_trade_with(tx, &input, true)?;
                    created += 1;
                }
            }
        }
        Ok((created, updated, assets_created, new_assets))
    })?;
    let (created, updated, assets_created, new_assets) = rec.value;
    // Códigos informados à mão: a próxima nota já vem resolvida.
    let made: HashMap<String, String> = choices
        .iter()
        .filter_map(|(s, c)| match c {
            Choice::New(a) => new_assets.get(&a.ticker).map(|id| (s.clone(), id.clone())),
            Choice::Asset(_) => None,
        })
        .collect();
    remember(ctx.conn(), &made)?;
    Ok(NoteCommitResult {
        created,
        updated,
        assets_created,
        skipped,
        change_set_id: rec.change_set_id,
    })
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn allocation_closes_the_total() {
        assert_eq!(allocate(118, &[285000, 7000, 160000]), vec![75, 1, 42]);
        assert_eq!(
            allocate(118, &[285000, 7000, 160000]).iter().sum::<i64>(),
            118
        );
        assert_eq!(allocate(8, &[0, 0, 160000]), vec![0, 0, 8]);
        assert_eq!(allocate(5, &[0, 0]), vec![0, 0]);
    }

    #[test]
    fn spec_name_and_kind() {
        assert_eq!(base_name("PETROBRAS PN N2"), "petrobras");
        assert_eq!(base_name("FII HGLG PAX HGLG11 CI"), "hglg pax hglg11");
        assert_eq!(kind_suffix("PETROBRAS PN N2"), Some("4"));
        assert_eq!(kind_suffix("VALE ON NM"), Some("3"));
        assert_eq!(kind_suffix("TAESA UNT N2"), Some("11"));
    }
}
