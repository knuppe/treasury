//! Filtros do e-mail de ingestão (docs/email-ingest.md). A ordem é sempre: pastas → bloqueios
//! globais → regras por prioridade (a primeira que casar decide). Sem regra, vale o padrão da
//! conta (ignorar). Bloqueio sempre vence permissão. Tudo aqui é puro: o botão "Testar" usa as
//! mesmas funções, sem importar nem apagar nada.

use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::import::text::fold;

/// Texto do catálogo com um argumento.
fn with(key: &str, name: &'static str, value: impl Into<String>) -> String {
    t_args(key, &args(&[(name, value.into().into())]))
}

/// Tamanho máximo de uma expressão regular (texto) e do autômato compilado.
const MAX_REGEX_LEN: usize = 300;
const REGEX_SIZE_LIMIT: usize = 1 << 20;

/// Pastas bloqueadas pela marca RFC 6154 (o nome nunca decide).
pub const DEFAULT_BLOCKED: [&str; 5] = ["\\Junk", "\\Trash", "\\Sent", "\\Drafts", "\\All"];

/// O que se sabe da mensagem para filtrar: os cabeçalhos (o corpo só depois do download).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MailEnvelope")]
#[serde(default)]
pub struct Envelope {
    /// Pasta (IMAP) ou subpasta (pasta local); `INBOX` na raiz.
    pub folder: String,
    /// Marcas RFC 6154 da pasta (`\Junk`, `\Trash`…).
    pub folder_flags: Vec<String>,
    /// Endereço do "De", em minúsculas.
    pub from: String,
    /// Remetente original de uma mensagem encaminhada (quando dá para saber).
    pub original_from: Option<String>,
    pub to: Vec<String>,
    /// Assunto decodificado (RFC 2047), sem `Fwd:`/`Enc:`/`RES:` na frente.
    pub subject: String,
    /// `YYYY-MM-DD`.
    pub date: Option<String>,
    #[ts(type = "number")]
    pub size: u64,
    /// Nomes dos anexos (vazio até o download, no IMAP).
    pub attachments: Vec<String>,
    /// Corpo como texto (só depois do download).
    pub body: Option<String>,
}

impl Envelope {
    /// O remetente que conta para as regras: o original, se a mensagem foi encaminhada.
    pub fn effective_from(&self) -> &str {
        self.original_from.as_deref().unwrap_or(&self.from)
    }
}

/// Assunto: contém termos (qualquer ou todos), não contém, ou expressão regular.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MailSubjectMatch")]
pub struct SubjectMatch {
    #[serde(default)]
    pub contains: Vec<String>,
    /// Todos os termos (padrão: qualquer um).
    #[serde(default)]
    pub all: bool,
    #[serde(default)]
    pub not_contains: Vec<String>,
    #[serde(default)]
    #[ts(optional)]
    pub regex: Option<String>,
}

/// Condição avançada (E/OU/NÃO sobre destinatário, anexo, tamanho, data e corpo).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export, rename = "MailCondition")]
pub enum Condition {
    All {
        of: Vec<Condition>,
    },
    Any {
        of: Vec<Condition>,
    },
    Not {
        cond: Box<Condition>,
    },
    /// Um destinatário casa com algum padrão (mesmos padrões do remetente).
    Recipient {
        patterns: Vec<String>,
    },
    /// Tem anexo (com um dos nomes ou extensões, se dados: `*.pdf`, `fatura*`).
    Attachment {
        names: Vec<String>,
    },
    /// Tamanho da mensagem, em KB.
    SizeAtMost {
        #[ts(type = "number")]
        kb: u64,
    },
    SizeAtLeast {
        #[ts(type = "number")]
        kb: u64,
    },
    /// Data da mensagem a partir de (`YYYY-MM-DD`).
    DateFrom {
        date: String,
    },
    DateUntil {
        date: String,
    },
    /// O corpo contém algum dos termos.
    BodyContains {
        terms: Vec<String>,
    },
}

/// Condições de uma regra.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MailRuleConditions")]
pub struct RuleConditions {
    /// Pastas em que a regra vale (vazio: todas as permitidas).
    #[serde(default)]
    pub folders: Vec<String>,
    /// Remetentes permitidos (vazio: qualquer um). Padrões: `nome@banco.com.br`,
    /// `banco.com.br` ou `@banco.com.br` (domínio), `*.itau.com.br` (subdomínios), curinga
    /// (`fatura*@*.com.br`), `/regex/` e `membros` (os endereços da família).
    #[serde(default)]
    pub from: Vec<String>,
    /// Remetentes bloqueados nesta regra.
    #[serde(default)]
    pub from_block: Vec<String>,
    #[serde(default)]
    pub subject: SubjectMatch,
    #[serde(default)]
    #[ts(optional)]
    pub advanced: Option<Condition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "MailRuleAction")]
pub enum RuleAction {
    /// Vai para a fila de revisão (padrão).
    Review,
    /// Aplica sozinho, quando o layout é determinístico, o remetente é autenticado e a prévia
    /// não tem aviso; senão, revisão.
    Auto,
    Ignore,
}

impl RuleAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Review => "review",
            Self::Auto => "auto",
            Self::Ignore => "ignore",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "auto" => Self::Auto,
            "ignore" => Self::Ignore,
            _ => Self::Review,
        }
    }
}

/// A regra como o filtro precisa dela.
#[derive(Debug, Clone)]
pub struct RuleSpec {
    pub id: String,
    pub name: String,
    pub conditions: RuleConditions,
    pub action: RuleAction,
}

/// Configuração da conta que o filtro usa.
#[derive(Debug, Clone, Default)]
pub struct AccountFilter {
    pub allowed_folders: Vec<String>,
    pub blocked_folders: Vec<String>,
    pub allow_senders: Vec<String>,
    pub block_senders: Vec<String>,
    pub family: Vec<String>,
    /// Sem regra casada: revisar (senão, ignorar).
    pub default_review: bool,
}

/// O que o filtro decidiu, com o motivo (para o "Testar" e a auditoria).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "MailDecision")]
pub struct Decision {
    /// Passou: vai ser baixada e processada.
    pub passed: bool,
    pub rule_id: Option<String>,
    pub rule_name: Option<String>,
    pub action: Option<RuleAction>,
    pub reason: String,
    /// A regra tem condição sobre o corpo: só decide de vez depois do download.
    pub needs_body: bool,
}

fn decision(passed: bool, reason: impl Into<String>) -> Decision {
    Decision {
        passed,
        rule_id: None,
        rule_name: None,
        action: None,
        reason: reason.into(),
        needs_body: false,
    }
}

/// Compila uma expressão regular com limite de tamanho; o erro diz onde está o problema.
pub fn compile_regex(pattern: &str) -> Result<Regex> {
    if pattern.len() > MAX_REGEX_LEN {
        return Err(Error::invalid(t_args(
            "nucleo-email-regex-erro-longa",
            &args(&[("max", MAX_REGEX_LEN.into())]),
        )));
    }
    RegexBuilder::new(pattern)
        .case_insensitive(true)
        .size_limit(REGEX_SIZE_LIMIT)
        .dfa_size_limit(REGEX_SIZE_LIMIT)
        .build()
        .map_err(|e| match e {
            regex::Error::Syntax(s) => {
                Error::invalid(with("nucleo-email-regex-erro-sintaxe", "erro", s))
            }
            regex::Error::CompiledTooBig(_) => Error::invalid(t("nucleo-email-regex-erro-grande")),
            other => Error::invalid(with(
                "nucleo-lancamentos-regra-erro-regex",
                "erro",
                other.to_string(),
            )),
        })
}

/// Curinga (`*` e `?`) em expressão regular inteira.
fn glob(pattern: &str) -> Option<Regex> {
    let mut re = String::from("^");
    for c in pattern.chars() {
        match c {
            '*' => re.push_str(".*"),
            '?' => re.push('.'),
            c => re.push_str(&regex::escape(&c.to_string())),
        }
    }
    re.push('$');
    RegexBuilder::new(&re).case_insensitive(true).build().ok()
}

fn domain_of(addr: &str) -> &str {
    addr.rsplit_once('@').map_or(addr, |(_, d)| d)
}

/// Um endereço casa com um padrão de remetente.
pub fn sender_matches(pattern: &str, addr: &str, family: &[String]) -> bool {
    let p = pattern.trim().to_ascii_lowercase();
    let a = addr.trim().to_ascii_lowercase();
    if p.is_empty() || a.is_empty() {
        return false;
    }
    if p == "membros" || p == "membros da familia" || p == "membros da família" {
        return family.iter().any(|f| f.trim().eq_ignore_ascii_case(&a));
    }
    if p.len() > 2 && p.starts_with('/') && p.ends_with('/') {
        return compile_regex(&pattern.trim()[1..pattern.trim().len() - 1])
            .map(|re| re.is_match(&a))
            .unwrap_or(false);
    }
    if let Some(sub) = p.strip_prefix("*.")
        && !sub.contains('*')
        && !sub.contains('@')
    {
        // Subdomínios: `*.itau.com.br` casa `x.itau.com.br`, não `itau.com.br`.
        return domain_of(&a).ends_with(&format!(".{sub}"));
    }
    if p.contains('*') || p.contains('?') {
        return glob(&p).is_some_and(|re| re.is_match(&a));
    }
    if let Some(d) = p.strip_prefix('@') {
        return domain_of(&a) == d;
    }
    if p.contains('@') {
        return p == a;
    }
    domain_of(&a) == p
}

fn any_sender(patterns: &[String], addr: &str, family: &[String]) -> bool {
    patterns.iter().any(|p| sender_matches(p, addr, family))
}

/// Uma pasta casa com um padrão: o nome (com curinga, `Bancos/*`) ou a marca (`\Junk`).
pub fn folder_matches(pattern: &str, folder: &str, flags: &[String]) -> bool {
    let p = pattern.trim();
    if p.starts_with('\\') {
        return flags.iter().any(|f| f.eq_ignore_ascii_case(p));
    }
    if p.eq_ignore_ascii_case("INBOX") {
        return folder.eq_ignore_ascii_case("INBOX");
    }
    if p.contains('*') || p.contains('?') {
        return glob(p).is_some_and(|re| re.is_match(folder));
    }
    p == folder
}

/// Tira `Fwd:`, `Enc:`, `RES:`, `Re:`, `TR:` e afins da frente do assunto.
pub fn clean_subject(s: &str) -> String {
    let mut t = s.trim().to_string();
    loop {
        let low = t.to_ascii_lowercase();
        let cut = [
            "fwd:",
            "fw:",
            "enc:",
            "res:",
            "re:",
            "tr:",
            "rv:",
            "encaminhada:",
            "encaminhado:",
        ]
        .iter()
        .find(|p| low.starts_with(*p))
        .map(|p| p.len());
        match cut {
            Some(n) => t = t[n..].trim_start().to_string(),
            None => break,
        }
    }
    t
}

fn subject_matches(m: &SubjectMatch, subject: &str) -> bool {
    let s = fold(subject);
    let has = |term: &String| {
        let t = fold(term);
        !t.is_empty() && s.contains(&t)
    };
    if !m.contains.is_empty() {
        let ok = if m.all {
            m.contains.iter().all(has)
        } else {
            m.contains.iter().any(has)
        };
        if !ok {
            return false;
        }
    }
    if m.not_contains.iter().any(has) {
        return false;
    }
    if let Some(re) = m.regex.as_deref().filter(|r| !r.trim().is_empty()) {
        return compile_regex(re).is_ok_and(|re| re.is_match(subject));
    }
    true
}

/// Resultado de uma condição: casa, não casa, ou depende do corpo (ainda não baixado).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tri {
    Yes,
    No,
    Unknown,
}

impl Tri {
    fn of(b: bool) -> Self {
        if b { Tri::Yes } else { Tri::No }
    }
}

fn eval(c: &Condition, env: &Envelope, family: &[String]) -> Tri {
    match c {
        Condition::All { of } => {
            let mut unknown = false;
            for x in of {
                match eval(x, env, family) {
                    Tri::No => return Tri::No,
                    Tri::Unknown => unknown = true,
                    Tri::Yes => {}
                }
            }
            if unknown { Tri::Unknown } else { Tri::Yes }
        }
        Condition::Any { of } => {
            let mut unknown = false;
            for x in of {
                match eval(x, env, family) {
                    Tri::Yes => return Tri::Yes,
                    Tri::Unknown => unknown = true,
                    Tri::No => {}
                }
            }
            if unknown { Tri::Unknown } else { Tri::No }
        }
        Condition::Not { cond } => match eval(cond, env, family) {
            Tri::Yes => Tri::No,
            Tri::No => Tri::Yes,
            Tri::Unknown => Tri::Unknown,
        },
        Condition::Recipient { patterns } => {
            Tri::of(env.to.iter().any(|t| any_sender(patterns, t, family)))
        }
        Condition::Attachment { names } => {
            if env.attachments.is_empty() && env.body.is_none() {
                // No IMAP, os anexos só aparecem depois do download.
                return Tri::Unknown;
            }
            Tri::of(if names.is_empty() {
                !env.attachments.is_empty()
            } else {
                env.attachments.iter().any(|a| {
                    names
                        .iter()
                        .any(|n| glob(n).is_some_and(|re| re.is_match(a)))
                })
            })
        }
        Condition::SizeAtMost { kb } => Tri::of(env.size <= kb * 1024),
        Condition::SizeAtLeast { kb } => Tri::of(env.size >= kb * 1024),
        Condition::DateFrom { date } => {
            Tri::of(env.date.as_deref().is_some_and(|d| d >= date.as_str()))
        }
        Condition::DateUntil { date } => {
            Tri::of(env.date.as_deref().is_some_and(|d| d <= date.as_str()))
        }
        Condition::BodyContains { terms } => match &env.body {
            None => Tri::Unknown,
            Some(b) => {
                let fb = fold(b);
                Tri::of(terms.iter().any(|t| {
                    let ft = fold(t);
                    !ft.is_empty() && fb.contains(&ft)
                }))
            }
        },
    }
}

/// Uma regra casa com a mensagem (`Unknown`: depende do corpo).
fn rule_matches(r: &RuleConditions, env: &Envelope, family: &[String]) -> Tri {
    if !r.folders.is_empty()
        && !r
            .folders
            .iter()
            .any(|f| folder_matches(f, &env.folder, &env.folder_flags))
    {
        return Tri::No;
    }
    let from = env.effective_from();
    if any_sender(&r.from_block, from, family) || any_sender(&r.from_block, &env.from, family) {
        return Tri::No;
    }
    if !r.from.is_empty() && !any_sender(&r.from, from, family) {
        return Tri::No;
    }
    if !subject_matches(&r.subject, &env.subject) {
        return Tri::No;
    }
    match &r.advanced {
        Some(c) => eval(c, env, family),
        None => Tri::Yes,
    }
}

/// Decide o que fazer com a mensagem. Com `needs_body`, baixe e chame de novo com o corpo.
pub fn decide(acc: &AccountFilter, rules: &[RuleSpec], env: &Envelope) -> Decision {
    // 1. Pastas: bloqueada sempre vence; fora das permitidas, nada.
    if acc
        .blocked_folders
        .iter()
        .any(|f| folder_matches(f, &env.folder, &env.folder_flags))
    {
        return decision(
            false,
            with(
                "nucleo-email-filtro-pasta-bloqueada",
                "pasta",
                env.folder.clone(),
            ),
        );
    }
    if !acc
        .allowed_folders
        .iter()
        .any(|f| folder_matches(f, &env.folder, &env.folder_flags))
    {
        return decision(
            false,
            with(
                "nucleo-email-filtro-pasta-fora",
                "pasta",
                env.folder.clone(),
            ),
        );
    }
    // 2. Bloqueios globais (do remetente que aparece e do original).
    let from = env.effective_from();
    if any_sender(&acc.block_senders, &env.from, &acc.family)
        || any_sender(&acc.block_senders, from, &acc.family)
    {
        return decision(
            false,
            with("nucleo-email-filtro-remetente-bloqueado", "remetente", from),
        );
    }
    if !acc.allow_senders.is_empty() && !any_sender(&acc.allow_senders, from, &acc.family) {
        return decision(
            false,
            with("nucleo-email-filtro-remetente-fora", "remetente", from),
        );
    }
    // 3. Regras por prioridade: a primeira que casar decide.
    for r in rules {
        match rule_matches(&r.conditions, env, &acc.family) {
            Tri::No => continue,
            Tri::Unknown => {
                return Decision {
                    passed: true,
                    rule_id: Some(r.id.clone()),
                    rule_name: Some(r.name.clone()),
                    action: Some(r.action),
                    reason: with("nucleo-email-filtro-regra-corpo", "regra", r.name.clone()),
                    needs_body: true,
                };
            }
            Tri::Yes => {
                return Decision {
                    passed: r.action != RuleAction::Ignore,
                    rule_id: Some(r.id.clone()),
                    rule_name: Some(r.name.clone()),
                    action: Some(r.action),
                    reason: with(
                        if r.action == RuleAction::Ignore {
                            "nucleo-email-filtro-regra-ignorar"
                        } else {
                            "nucleo-email-filtro-regra"
                        },
                        "regra",
                        r.name.clone(),
                    ),
                    needs_body: false,
                };
            }
        }
    }
    if acc.default_review {
        Decision {
            action: Some(RuleAction::Review),
            ..decision(true, t("nucleo-email-filtro-padrao-revisar"))
        }
    } else {
        decision(false, t("nucleo-email-filtro-padrao-ignorar"))
    }
}

/// Confere padrões e expressões de uma regra ao salvar (erro com a posição).
pub fn validate(c: &RuleConditions) -> Result<()> {
    let check_patterns = |ps: &[String]| -> Result<()> {
        for p in ps {
            let t = p.trim();
            if t.len() > 2 && t.starts_with('/') && t.ends_with('/') {
                compile_regex(&t[1..t.len() - 1])?;
            }
        }
        Ok(())
    };
    check_patterns(&c.from)?;
    check_patterns(&c.from_block)?;
    if let Some(r) = c.subject.regex.as_deref().filter(|r| !r.trim().is_empty()) {
        compile_regex(r)?;
    }
    fn walk(c: &Condition, depth: usize) -> Result<()> {
        if depth > 6 {
            return Err(Error::invalid(t("nucleo-email-filtro-erro-aninhadas")));
        }
        match c {
            Condition::All { of } | Condition::Any { of } => {
                for x in of {
                    walk(x, depth + 1)?;
                }
                Ok(())
            }
            Condition::Not { cond } => walk(cond, depth + 1),
            Condition::DateFrom { date } | Condition::DateUntil { date } => {
                crate::time::parse_date(date).map(|_| ())
            }
            _ => Ok(()),
        }
    }
    if let Some(a) = &c.advanced {
        walk(a, 0)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(from: &str, subject: &str) -> Envelope {
        Envelope {
            folder: "INBOX".into(),
            from: from.into(),
            subject: subject.into(),
            size: 10_000,
            ..Default::default()
        }
    }

    fn acc() -> AccountFilter {
        AccountFilter {
            allowed_folders: vec!["INBOX".into(), "Bancos/*".into()],
            blocked_folders: DEFAULT_BLOCKED.iter().map(|s| s.to_string()).collect(),
            family: vec!["ana@gmail.com".into()],
            ..Default::default()
        }
    }

    fn rule(id: &str, c: RuleConditions, action: RuleAction) -> RuleSpec {
        RuleSpec {
            id: id.into(),
            name: id.into(),
            conditions: c,
            action,
        }
    }

    #[test]
    fn sender_patterns() {
        let fam = vec!["ana@gmail.com".to_string()];
        assert!(sender_matches(
            "faturas@itau.com.br",
            "Faturas@Itau.com.br",
            &fam
        ));
        assert!(sender_matches("itau.com.br", "x@itau.com.br", &fam));
        assert!(!sender_matches("itau.com.br", "x@mail.itau.com.br", &fam));
        assert!(sender_matches("@itau.com.br", "x@itau.com.br", &fam));
        assert!(sender_matches("*.itau.com.br", "x@mail.itau.com.br", &fam));
        assert!(!sender_matches("*.itau.com.br", "x@itau.com.br", &fam));
        assert!(!sender_matches(
            "*.itau.com.br",
            "x@itau.com.br.evil.com",
            &fam
        ));
        assert!(sender_matches(
            "fatura*@*.com.br",
            "fatura-cartao@nubank.com.br",
            &fam
        ));
        assert!(sender_matches(
            "/^nao-?responda@/",
            "naoresponda@banco.com",
            &fam
        ));
        assert!(sender_matches("membros", "ana@gmail.com", &fam));
        assert!(!sender_matches("membros", "bruno@gmail.com", &fam));
    }

    #[test]
    fn order_folders_blocks_rules_and_default() {
        let a = acc();
        let nubank = rule(
            "nubank",
            RuleConditions {
                from: vec!["nubank.com.br".into()],
                subject: SubjectMatch {
                    contains: vec!["fatura".into()],
                    ..Default::default()
                },
                ..Default::default()
            },
            RuleAction::Review,
        );
        let rules = vec![nubank];
        // Casa (sem diferenciar acento nem maiúscula; sem o "Fwd:").
        let mut e = env(
            "todomundo@nubank.com.br",
            &clean_subject("Fwd: Sua FATURA chegou"),
        );
        let d = decide(&a, &rules, &e);
        assert!(d.passed && d.rule_id.as_deref() == Some("nubank"), "{d:?}");
        // Pasta bloqueada pela marca, mesmo com nome inocente.
        e.folder = "Lixo".into();
        e.folder_flags = vec!["\\Trash".into()];
        assert!(!decide(&a, &rules, &e).passed);
        // Subpasta com curinga.
        e.folder = "Bancos/Nubank".into();
        e.folder_flags = vec![];
        assert!(decide(&a, &rules, &e).passed);
        // Pasta fora das permitidas.
        e.folder = "Promoções".into();
        assert!(!decide(&a, &rules, &e).passed);
        // Sem regra: padrão ignora; com revisão, passa.
        let other = env("loja@exemplo.com", "Promoção");
        assert!(!decide(&a, &rules, &other).passed);
        let mut a2 = acc();
        a2.default_review = true;
        assert!(decide(&a2, &rules, &other).passed);
        // Bloqueio global vence a regra.
        let mut a3 = acc();
        a3.block_senders = vec!["*.com.br".into()];
        let e = env("todomundo@x.nubank.com.br", "Sua fatura");
        assert!(!decide(&a3, &rules, &e).passed);
    }

    #[test]
    fn forwarded_mail_matches_the_original_sender() {
        let a = acc();
        let rules = vec![rule(
            "itau",
            RuleConditions {
                from: vec!["itau.com.br".into()],
                ..Default::default()
            },
            RuleAction::Auto,
        )];
        let e = Envelope {
            original_from: Some("faturas@itau.com.br".into()),
            ..env("ana@gmail.com", "Fatura")
        };
        assert!(decide(&a, &rules, &e).passed);
    }

    #[test]
    fn advanced_conditions_and_body() {
        let a = acc();
        let rules = vec![rule(
            "pix",
            RuleConditions {
                advanced: Some(Condition::All {
                    of: vec![
                        Condition::Not {
                            cond: Box::new(Condition::SizeAtLeast { kb: 5_000 }),
                        },
                        Condition::BodyContains {
                            terms: vec!["pix recebido".into()],
                        },
                    ],
                }),
                ..Default::default()
            },
            RuleAction::Review,
        )];
        let mut e = env("banco@inter.co", "Aviso");
        let d = decide(&a, &rules, &e);
        assert!(d.passed && d.needs_body);
        e.body = Some("Você recebeu um PIX recebido de R$ 10".into());
        let d = decide(&a, &rules, &e);
        assert!(d.passed && !d.needs_body);
        e.body = Some("nada".into());
        assert!(!decide(&a, &rules, &e).passed);
    }

    #[test]
    fn regex_errors_show_where() {
        let e = validate(&RuleConditions {
            subject: SubjectMatch {
                regex: Some("fatura (".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap_err();
        assert!(
            e.to_string().contains("regex parse error") || e.to_string().contains("inválida"),
            "{e}"
        );
        assert!(compile_regex(&"a".repeat(301)).is_err());
        assert_eq!(clean_subject("RES: ENC: Fwd: Sua fatura"), "Sua fatura");
    }

    /// Os motivos das decisões saem no idioma da instalação (o nome da regra fica).
    #[test]
    fn decisions_follow_the_language() {
        use crate::i18n::{Locale, set_current};
        set_current(Locale::En);
        let mut a = acc();
        a.block_senders = vec!["spam@x.com".into()];
        let d = decide(&a, &[], &env("spam@x.com", "Oi"));
        assert_eq!(d.reason, "blocked sender: spam@x.com");
        let d = decide(&acc(), &[], &env("faturas@itau.com.br", "Fatura"));
        assert_eq!(d.reason, "no rule matched: account default (ignore)");
        let r = rule(
            "Itaú",
            RuleConditions {
                from: vec!["itau.com.br".into()],
                ..Default::default()
            },
            RuleAction::Ignore,
        );
        let d = decide(&acc(), &[r], &env("faturas@itau.com.br", "Fatura"));
        assert_eq!(d.reason, "rule \"Itaú\": ignore");
        set_current(Locale::PtBr);
    }
}
