//! Idiomas (docs/i18n-design.md): os catálogos Fluent de `locales/` embutidos na compilação, o
//! idioma da instalação (campo `locale` do `config.toml`, fora do cofre) e a formatação de
//! números, datas e dinheiro por idioma para os textos que o próprio núcleo monta.
//!
//! O idioma é por instalação: a interface, o MCP, as notificações e as rotinas usam o mesmo.
//! Reserva: pt-PT → pt-BR; en → pt-BR; es → en → pt-BR. `pt` é o mesmo que pt-BR.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::RwLock;

use chrono::{Datelike, NaiveDate};
use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use unic_langid::LanguageIdentifier;

use crate::error::{Error, Result};

/// Idiomas do seletor, na ordem em que aparecem.
pub const AVAILABLE: [Locale; 4] = [Locale::PtBr, Locale::En, Locale::Es, Locale::PtPt];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Locale {
    #[serde(rename = "pt-BR")]
    PtBr,
    #[serde(rename = "en")]
    En,
    #[serde(rename = "es")]
    Es,
    /// Variante opcional: o que faltar cai para pt-BR.
    #[serde(rename = "pt-PT")]
    PtPt,
}

impl Locale {
    pub fn code(self) -> &'static str {
        match self {
            Self::PtBr => "pt-BR",
            Self::En => "en",
            Self::Es => "es",
            Self::PtPt => "pt-PT",
        }
    }

    /// Lê um código (`pt`, `pt_BR`, `en-US`, `es-419`…); desconhecido cai em `None`.
    pub fn parse(code: &str) -> Option<Self> {
        let c = code.trim().replace('_', "-").to_ascii_lowercase();
        let lang = c.split('-').next().unwrap_or("");
        match (lang, c.as_str()) {
            (_, "pt-pt") => Some(Self::PtPt),
            ("pt", _) => Some(Self::PtBr),
            ("en", _) => Some(Self::En),
            ("es", _) => Some(Self::Es),
            _ => None,
        }
    }

    /// O idioma e as reservas dele, na ordem.
    pub fn chain(self) -> &'static [Locale] {
        match self {
            Self::PtBr => &[Self::PtBr],
            Self::PtPt => &[Self::PtPt, Self::PtBr],
            Self::En => &[Self::En, Self::PtBr],
            Self::Es => &[Self::Es, Self::En, Self::PtBr],
        }
    }

    fn langid(self) -> LanguageIdentifier {
        self.code().parse().expect("código de idioma válido")
    }
}

static CURRENT: RwLock<Locale> = RwLock::new(Locale::PtBr);

#[cfg(test)]
thread_local! {
    /// Nos testes, o idioma é por thread (cada teste roda na sua): trocar num não muda os outros.
    static TEST_CURRENT: std::cell::Cell<Locale> = const { std::cell::Cell::new(Locale::PtBr) };
}

/// Idioma atual da instalação.
pub fn current() -> Locale {
    #[cfg(test)]
    {
        TEST_CURRENT.with(|c| c.get())
    }
    #[cfg(not(test))]
    {
        *CURRENT.read().unwrap_or_else(|p| p.into_inner())
    }
}

/// Troca o idioma atual (sem gravar; ver [`save`]).
pub fn set_current(l: Locale) {
    #[cfg(test)]
    TEST_CURRENT.with(|c| c.set(l));
    *CURRENT.write().unwrap_or_else(|p| p.into_inner()) = l;
}

/// Lê o idioma do `config.toml` (sem o campo, ou com um código desconhecido: pt-BR).
pub fn load(config_path: &std::path::Path) -> Locale {
    crate::config::Config::load(config_path)
        .ok()
        .and_then(|c| c.locale)
        .and_then(|c| Locale::parse(&c))
        .unwrap_or(Locale::PtBr)
}

/// Grava o idioma no `config.toml` e passa a usá-lo.
pub fn save(config_path: &std::path::Path, l: Locale) -> Result<()> {
    let mut c = crate::config::Config::load(config_path)?;
    c.locale = Some(l.code().into());
    c.save(config_path)?;
    set_current(l);
    Ok(())
}

/// Argumentos de uma mensagem.
pub type Args = HashMap<Cow<'static, str>, FluentValue<'static>>;

/// Monta os argumentos: `args(&[("nome", "Ana".into()), ("dias", 3.into())])`.
pub fn args(list: &[(&'static str, FluentValue<'static>)]) -> Args {
    list.iter()
        .map(|(k, v)| (Cow::Borrowed(*k), v.clone()))
        .collect()
}

/// Texto da chave no idioma `l` (com as reservas). Sem a chave em nenhum: a própria chave.
/// Uma mensagem com erro ao montar (argumento faltando, referência quebrada) passa para a
/// reserva seguinte.
pub fn tr(l: Locale, key: &str, args: Option<&Args>) -> String {
    let args = args.map(|a| {
        let mut fa = FluentArgs::with_capacity(a.len());
        for (k, v) in a {
            fa.set(k.clone(), v.clone());
        }
        fa
    });
    for loc in l.chain().iter().filter(|loc| has_key(**loc, key)) {
        if let Some(s) = format(*loc, key, args.as_ref()) {
            return s;
        }
    }
    key.to_string()
}

/// Uma mensagem (`chave` ou `chave.atributo`) só no catálogo de `l`, sem erros ao montar.
fn format(l: Locale, key: &str, args: Option<&FluentArgs>) -> Option<String> {
    let bundle = bundle(l);
    let pattern = match key.split_once('.') {
        Some((msg, attr)) => bundle.get_message(msg)?.get_attribute(attr)?.value(),
        None => bundle.get_message(key)?.value()?,
    };
    let mut errors = Vec::new();
    let s = bundle.format_pattern(pattern, args, &mut errors);
    errors.is_empty().then(|| s.into_owned())
}

/// Catálogo Fluent de um idioma, montado uma vez a partir dos `.ftl` embutidos.
fn bundle(l: Locale) -> &'static FluentBundle<FluentResource> {
    use std::sync::OnceLock;
    static BUNDLES: OnceLock<HashMap<Locale, FluentBundle<FluentResource>>> = OnceLock::new();
    &BUNDLES.get_or_init(|| {
        AVAILABLE
            .iter()
            .map(|loc| (*loc, build_bundle(*loc)))
            .collect()
    })[&l]
}

/// Monta o catálogo de um idioma. Um trecho com erro de sintaxe fica de fora (o resto do
/// arquivo entra); o teste `catalogs_parse_cleanly` não deixa isso chegar a uma versão.
fn build_bundle(l: Locale) -> FluentBundle<FluentResource> {
    let mut bundle = FluentBundle::new_concurrent(vec![l.langid()]);
    // Sem os marcadores de isolamento Unicode em volta dos argumentos (a interface e o MCP
    // mostram texto puro).
    bundle.set_use_isolating(false);
    for (file, src) in sources(l) {
        let res = FluentResource::try_new(src.to_string()).unwrap_or_else(|(res, errors)| {
            tracing::warn!(
                "catálogo {}/{file}: {} erros de sintaxe",
                l.code(),
                errors.len()
            );
            res
        });
        if let Err(errors) = bundle.add_resource(res) {
            tracing::warn!(
                "catálogo {}/{file}: {} chaves repetidas",
                l.code(),
                errors.len()
            );
        }
    }
    bundle
}

/// A chave existe no catálogo deste idioma (sem reserva)?
pub fn has_key(l: Locale, key: &str) -> bool {
    catalog_keys(l).contains(key)
}

/// Texto da chave no idioma atual.
pub fn t(key: &str) -> String {
    tr(current(), key, None)
}

/// Texto da chave com argumentos no idioma atual.
pub fn t_args(key: &str, a: &Args) -> String {
    tr(current(), key, Some(a))
}

// ---------------------------------------------------------------------------
// Catálogos como texto (paridade e testes)

/// Chaves (mensagens) de um idioma, lidas dos `.ftl` embutidos.
pub fn catalog_keys(l: Locale) -> &'static std::collections::BTreeSet<String> {
    use std::collections::BTreeSet;
    use std::sync::OnceLock;
    static CACHE: OnceLock<HashMap<Locale, BTreeSet<String>>> = OnceLock::new();
    &CACHE.get_or_init(|| {
        AVAILABLE
            .iter()
            .map(|loc| {
                let keys = sources(*loc)
                    .iter()
                    .flat_map(|(_, src)| message_ids(src))
                    .collect();
                (*loc, keys)
            })
            .collect()
    })[&l]
}

/// Ids das mensagens de um `.ftl` (linhas `chave = …` no começo da linha).
pub fn message_ids(src: &str) -> Vec<String> {
    src.lines()
        .filter(|l| l.chars().next().is_some_and(|c| c.is_ascii_alphabetic()))
        .filter_map(|l| l.split_once('=').map(|(k, _)| k.trim().to_string()))
        .filter(|k| {
            !k.is_empty()
                && k.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
        .collect()
}

/// Os `.ftl` de um idioma (nome do arquivo, conteúdo), embutidos para os testes de paridade.
pub fn sources(l: Locale) -> Vec<(&'static str, &'static str)> {
    const ALL: &[(&str, &str, &str)] = include!(concat!(env!("OUT_DIR"), "/locales.rs"));
    ALL.iter()
        .filter(|(code, _, _)| *code == l.code())
        .map(|(_, f, s)| (*f, *s))
        .collect()
}

// ---------------------------------------------------------------------------
// Formatação (a mesma tabela de `lib/format.ts`)

struct Fmt {
    thousands: &'static str,
    decimal: char,
    /// pt-PT: o separador de milhar só a partir de 5 dígitos.
    min_group: usize,
}

fn fmt_of(l: Locale) -> Fmt {
    match l {
        Locale::PtBr | Locale::Es => Fmt {
            thousands: ".",
            decimal: ',',
            min_group: 4,
        },
        Locale::PtPt => Fmt {
            thousands: "\u{202F}",
            decimal: ',',
            min_group: 5,
        },
        Locale::En => Fmt {
            thousands: ",",
            decimal: '.',
            min_group: 4,
        },
    }
}

/// Inteiro com o separador de milhar do idioma.
fn group(l: Locale, digits: &str) -> String {
    let f = fmt_of(l);
    if digits.len() < f.min_group {
        return digits.to_string();
    }
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push_str(f.thousands);
        }
        out.push(c);
    }
    out
}

/// Valor em centavos com 2 casas, sem moeda, no idioma (`1.234,56`, `1,234.56`).
pub fn number(l: Locale, cents: i64) -> String {
    let neg = cents < 0;
    let abs = cents.unsigned_abs();
    let s = format!(
        "{}{}{:02}",
        group(l, &(abs / 100).to_string()),
        fmt_of(l).decimal,
        abs % 100
    );
    if neg { format!("\u{2212}{s}") } else { s }
}

/// Valor na menor unidade com `units` casas (0 no iene, 3 no dinar do Kuwait), no idioma.
fn with_units(l: Locale, minor: u64, units: u32) -> String {
    let pow = 10u64.pow(units);
    let int = group(l, &(minor / pow).to_string());
    if units == 0 {
        int
    } else {
        let frac = minor % pow;
        format!("{int}{}{frac:0w$}", fmt_of(l).decimal, w = units as usize)
    }
}

/// Dinheiro no idioma: o símbolo da moeda "da casa" do idioma, senão o código ISO; as casas
/// decimais são as da moeda.
pub fn money(l: Locale, cents: i64, currency: &str) -> String {
    let neg = cents < 0;
    let n = with_units(l, cents.unsigned_abs(), crate::money::minor_units(currency));
    let body = match (l, currency) {
        (Locale::PtBr, "BRL") => format!("R$ {n}"),
        (Locale::En, "USD") => format!("${n}"),
        (Locale::En, "BRL") => format!("R${n}"),
        (Locale::Es, "USD") => format!("US$ {n}"),
        (Locale::PtPt, "EUR") => format!("{n} €"),
        (Locale::En, "EUR") => format!("€{n}"),
        (Locale::Es, "EUR") => format!("{n} €"),
        _ => format!("{currency} {n}"),
    };
    if neg { format!("\u{2212}{body}") } else { body }
}

/// Data no idioma: `14/10/2026` (pt, es) ou `Oct 14, 2026` (en).
pub fn date(l: Locale, d: NaiveDate) -> String {
    match l {
        Locale::En => {
            let m = month_names(l, true);
            format!("{} {}, {}", m[d.month0() as usize], d.day(), d.year())
        }
        _ => d.format("%d/%m/%Y").to_string(),
    }
}

/// Nomes dos meses do catálogo (`comum-meses`, `comum-meses-curtos`).
pub fn month_names(l: Locale, short: bool) -> Vec<String> {
    let key = if short {
        "comum-meses-curtos"
    } else {
        "comum-meses"
    };
    tr(l, key, None)
        .split(',')
        .map(|s| s.trim().to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// Comandos (`system.locale`, `system.set_locale`), valem sem cofre

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct LocaleOption {
    pub code: Locale,
    /// O nome na própria língua ("English", "Español").
    pub name: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct LocaleState {
    pub locale: Locale,
    /// Já foi escolhido (no `config.toml`); sem isso, a primeira tela pergunta.
    pub chosen: bool,
    pub available: Vec<LocaleOption>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct SetLocaleArgs {
    /// `pt-BR`, `pt`, `en`, `es` ou `pt-PT`.
    pub locale: String,
}

pub fn state(config_path: &std::path::Path) -> LocaleState {
    let chosen = crate::config::Config::load(config_path)
        .ok()
        .and_then(|c| c.locale)
        .is_some_and(|c| Locale::parse(&c).is_some());
    LocaleState {
        locale: current(),
        chosen,
        available: AVAILABLE
            .iter()
            .map(|l| LocaleOption {
                code: *l,
                name: tr(*l, &format!("comum-idioma-{}", l.code()), None),
            })
            .collect(),
    }
}

pub fn set(config_path: &std::path::Path, a: &SetLocaleArgs) -> Result<LocaleState> {
    let l = Locale::parse(&a.locale)
        .ok_or_else(|| Error::invalid(format!("idioma desconhecido: {}", a.locale)))?;
    save(config_path, l)?;
    Ok(state(config_path))
}

#[cfg(test)]
mod tests;
