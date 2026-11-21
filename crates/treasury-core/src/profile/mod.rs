//! Perfil de quem usa o app (docs/profile.md): nível de experiência, tom da linguagem,
//! objetivos e tipo de renda. Define o que a interface mostra por padrão (o resto fica a um
//! clique) e como o agente fala. Fica nos ajustes do cofre (`user_profile`).
//!
//! - Cofre novo, vazio e sem perfil: o app abre o questionário de boas-vindas
//!   (`onboarding_pending`), que dá para pular direto para o modo avançado.
//! - Cofre com dados e sem perfil (quem já usava o app): avançado, sem questionário.
//! - O nível sai de uma pontuação simples das respostas ([`score`]) e muda quando a pessoa
//!   quiser, em Ajustes › Perfil.

pub mod api;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};

const KEY: &str = "user_profile";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "ProfileLevel")]
pub enum Level {
    Iniciante,
    Intermediario,
    Avancado,
}

impl Level {
    /// Nome do nível no idioma da instalação.
    pub fn label(self) -> String {
        t(match self {
            Level::Iniciante => "nucleo-perfil-nivel-iniciante",
            Level::Intermediario => "nucleo-perfil-nivel-intermediario",
            Level::Avancado => "nucleo-perfil-nivel-avancado",
        })
    }
}

/// Como o app (e o agente) falam.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "ProfileTone")]
pub enum Tone {
    /// Bem simples, com explicações e exemplos do dia a dia.
    Simples,
    /// Claro, sem infantilizar.
    Equilibrado,
    /// Detalhado e técnico.
    Tecnico,
}

/// O tom que acompanha cada nível.
pub fn tone_for(l: Level) -> Tone {
    Tone::for_level(l)
}

impl Tone {
    fn for_level(l: Level) -> Tone {
        match l {
            Level::Iniciante => Tone::Simples,
            Level::Intermediario => Tone::Equilibrado,
            Level::Avancado => Tone::Tecnico,
        }
    }
}

/// De onde veio o nível.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename = "ProfileSource")]
pub enum Source {
    /// Respostas do questionário.
    Questionario,
    /// Pulou o questionário ("já sei o que quero").
    Pulou,
    /// Escolheu o nível em Ajustes.
    Escolha,
    /// Cofre de antes do perfil: avançado, sem perguntar.
    Padrao,
    /// Membro que ainda não fez o questionário: vale o perfil da família.
    Familia,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "UserProfile")]
pub struct Profile {
    pub level: Level,
    pub tone: Tone,
    pub source: Source,
    /// Objetivos escolhidos no questionário (ids das opções da pergunta `objetivo`).
    pub goals: Vec<String>,
    /// Tipo de renda principal (id da opção da pergunta `renda`), se informado.
    pub income: Option<String>,
    /// Respostas brutas: id da pergunta → ids das opções.
    pub answers: BTreeMap<String, Vec<String>>,
    /// O questionário ainda não foi feito nem pulado (cofre novo).
    pub onboarding_pending: bool,
    pub updated_at: Option<String>,
}

/// Uma opção de resposta.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "ProfileOption")]
pub struct QOption {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, rename = "ProfileQuestion")]
pub struct Question {
    pub id: String,
    pub title: String,
    /// Explicação curta abaixo da pergunta (opcional).
    pub hint: Option<String>,
    /// Aceita mais de uma opção.
    pub multiple: bool,
    pub options: Vec<QOption>,
}

/// Pontos de cada opção (0 = iniciante, 2 = avançado). Perguntas sem pontos só dão contexto.
type Spec = (
    &'static str,
    &'static str,
    Option<&'static str>,
    bool,
    &'static [(&'static str, &'static str, Option<f32>)],
);

/// (id, chave do título, chave da dica, múltipla, opções (id, chave do rótulo, pontos)): os textos
/// saem do catálogo (`nucleo-perfil.ftl`) no idioma da instalação.
const QUESTIONS: &[Spec] = &[
    (
        "confianca",
        "nucleo-perfil-confianca",
        None,
        false,
        &[
            ("nada", "nucleo-perfil-confianca-nada", Some(0.0)),
            (
                "mais_ou_menos",
                "nucleo-perfil-confianca-mais-ou-menos",
                Some(0.7),
            ),
            ("bastante", "nucleo-perfil-confianca-bastante", Some(1.4)),
            ("muito", "nucleo-perfil-confianca-muito", Some(2.0)),
        ],
    ),
    (
        "controle",
        "nucleo-perfil-controle",
        None,
        false,
        &[
            ("nunca", "nucleo-perfil-controle-nunca", Some(0.0)),
            ("as_vezes", "nucleo-perfil-controle-as-vezes", Some(0.5)),
            ("planilha", "nucleo-perfil-controle-planilha", Some(1.3)),
            ("detalhado", "nucleo-perfil-controle-detalhado", Some(2.0)),
        ],
    ),
    (
        "objetivo",
        "nucleo-perfil-objetivo",
        Some("nucleo-perfil-objetivo-dica"),
        true,
        &[
            ("onde_vai", "nucleo-perfil-objetivo-onde-vai", Some(0.0)),
            ("reserva", "nucleo-perfil-objetivo-reserva", Some(0.4)),
            ("dividas", "nucleo-perfil-objetivo-dividas", Some(0.8)),
            ("objetivo", "nucleo-perfil-objetivo-objetivo", Some(1.0)),
            ("organizar", "nucleo-perfil-objetivo-organizar", Some(1.5)),
            ("investir", "nucleo-perfil-objetivo-investir", Some(2.0)),
        ],
    ),
    (
        "reserva",
        "nucleo-perfil-reserva",
        None,
        false,
        &[
            ("nao_sei", "nucleo-perfil-reserva-nao-sei", Some(0.0)),
            ("nao_tenho", "nucleo-perfil-reserva-nao-tenho", Some(0.5)),
            ("um_pouco", "nucleo-perfil-reserva-um-pouco", Some(1.0)),
            ("suficiente", "nucleo-perfil-reserva-suficiente", Some(2.0)),
        ],
    ),
    (
        "renda",
        "nucleo-perfil-renda",
        Some("nucleo-perfil-renda-dica"),
        false,
        &[
            ("fixa", "nucleo-perfil-renda-fixa", None),
            ("variavel", "nucleo-perfil-renda-variavel", None),
            ("beneficios", "nucleo-perfil-renda-beneficios", None),
            ("mista", "nucleo-perfil-renda-mista", None),
            ("nao_informar", "nucleo-perfil-renda-nao-informar", None),
        ],
    ),
    (
        "termos",
        "nucleo-perfil-termos",
        None,
        false,
        &[
            ("perdido", "nucleo-perfil-termos-perdido", Some(0.0)),
            (
                "mais_ou_menos",
                "nucleo-perfil-termos-mais-ou-menos",
                Some(1.0),
            ),
            ("entendo", "nucleo-perfil-termos-entendo", Some(2.0)),
        ],
    ),
    (
        "tom",
        "nucleo-perfil-tom",
        None,
        false,
        &[
            ("simples", "nucleo-perfil-tom-simples", Some(0.0)),
            ("equilibrado", "nucleo-perfil-tom-equilibrado", Some(1.0)),
            ("tecnico", "nucleo-perfil-tom-tecnico", Some(2.0)),
        ],
    ),
    // Linha da vida (docs/life-expectancy.md §8): sem pontos, só no questionário da família;
    // "Sim" liga a função.
    (
        LONGEVITY_QUESTION,
        "nucleo-longevidade-pergunta",
        Some("nucleo-longevidade-pergunta-dica"),
        false,
        &[
            ("yes", "nucleo-longevidade-pergunta-sim", None),
            ("no", "nucleo-longevidade-pergunta-nao", None),
            ("later", "nucleo-longevidade-pergunta-depois", None),
        ],
    ),
];

/// Pergunta da linha da vida: a chave é da família, por isso não entra no questionário de um
/// membro.
pub const LONGEVITY_QUESTION: &str = "longevity";

/// Média abaixo disso: iniciante; abaixo de [`ADVANCED_FROM`]: intermediário.
const INTERMEDIATE_FROM: f32 = 0.8;
const ADVANCED_FROM: f32 = 1.5;

/// As perguntas do questionário da família.
pub fn questions() -> Vec<Question> {
    questions_for(false)
}

/// As perguntas; no questionário de um membro (`member`), sem as que são da família.
pub fn questions_for(member: bool) -> Vec<Question> {
    QUESTIONS
        .iter()
        .filter(|q| !(member && q.0 == LONGEVITY_QUESTION))
        .map(|(id, title, hint, multiple, opts)| Question {
            id: (*id).into(),
            title: t(title),
            hint: hint.map(t),
            multiple: *multiple,
            options: opts
                .iter()
                .map(|(oid, label, _)| QOption {
                    id: (*oid).into(),
                    label: t(label),
                })
                .collect(),
        })
        .collect()
}

/// Confere as respostas (perguntas e opções conhecidas, uma só onde não é múltipla).
pub fn validate(answers: &BTreeMap<String, Vec<String>>) -> Result<()> {
    for (q, picked) in answers {
        let Some((_, _, _, multiple, opts)) = QUESTIONS.iter().find(|s| s.0 == q) else {
            return Err(Error::invalid(t_args(
                "nucleo-perfil-erro-pergunta",
                &args(&[("pergunta", q.clone().into())]),
            )));
        };
        if !multiple && picked.len() > 1 {
            return Err(Error::invalid(t_args(
                "nucleo-perfil-erro-uma-resposta",
                &args(&[("pergunta", q.clone().into())]),
            )));
        }
        for p in picked {
            if !opts.iter().any(|o| o.0 == p) {
                return Err(Error::invalid(t_args(
                    "nucleo-perfil-erro-opcao",
                    &args(&[("pergunta", q.clone().into()), ("opcao", p.clone().into())]),
                )));
            }
        }
    }
    Ok(())
}

/// Nível e tom a partir das respostas: média dos pontos das perguntas respondidas (na de
/// objetivos, a média das escolhidas). Sem nenhuma resposta pontuada: intermediário.
pub fn score(answers: &BTreeMap<String, Vec<String>>) -> (Level, Tone) {
    let mut values = vec![];
    for (id, _, _, _, opts) in QUESTIONS {
        let Some(picked) = answers.get(*id) else {
            continue;
        };
        let pts: Vec<f32> = picked
            .iter()
            .filter_map(|p| opts.iter().find(|o| o.0 == p).and_then(|o| o.2))
            .collect();
        if !pts.is_empty() {
            values.push(pts.iter().sum::<f32>() / pts.len() as f32);
        }
    }
    let level = if values.is_empty() {
        Level::Intermediario
    } else {
        let avg = values.iter().sum::<f32>() / values.len() as f32;
        if avg < INTERMEDIATE_FROM {
            Level::Iniciante
        } else if avg < ADVANCED_FROM {
            Level::Intermediario
        } else {
            Level::Avancado
        }
    };
    let tone = match answers
        .get("tom")
        .and_then(|t| t.first())
        .map(String::as_str)
    {
        Some("simples") => Tone::Simples,
        Some("equilibrado") => Tone::Equilibrado,
        Some("tecnico") => Tone::Tecnico,
        _ => Tone::for_level(level),
    };
    (level, tone)
}

fn has_data(conn: &Connection) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM account) OR EXISTS (SELECT 1 FROM txn)",
        [],
        |r| r.get(0),
    )?)
}

/// O perfil salvo; sem ele, o padrão (avançado para quem já tem dados, questionário pendente
/// para um cofre vazio).
pub fn get(conn: &Connection) -> Result<Profile> {
    if let Some(p) = crate::settings::get::<Profile>(conn, KEY)? {
        return Ok(p);
    }
    Ok(Profile {
        level: Level::Avancado,
        tone: Tone::Tecnico,
        source: Source::Padrao,
        goals: vec![],
        income: None,
        answers: BTreeMap::new(),
        onboarding_pending: !has_data(conn)?,
        updated_at: None,
    })
}

pub fn save(conn: &Connection, p: &Profile) -> Result<()> {
    crate::settings::set(conn, KEY, p)
}

fn member_key(member_id: &str) -> String {
    format!("{KEY}:{member_id}")
}

/// O perfil de um membro da família (cada um faz o próprio questionário); sem ele, o da
/// família. `None`: o perfil da família (quem administra o cofre).
pub fn get_for(conn: &Connection, member_id: Option<&str>) -> Result<Profile> {
    let Some(m) = member_id else {
        return get(conn);
    };
    crate::household::get_member(conn, m)?;
    if let Some(p) = crate::settings::get::<Profile>(conn, &member_key(m))? {
        return Ok(p);
    }
    let mut p = get(conn)?;
    p.source = Source::Familia;
    p.answers = BTreeMap::new();
    p.goals = vec![];
    p.income = None;
    p.onboarding_pending = false;
    p.updated_at = None;
    Ok(p)
}

pub fn save_for(conn: &Connection, member_id: Option<&str>, p: &Profile) -> Result<()> {
    match member_id {
        None => save(conn, p),
        Some(m) => {
            crate::household::get_member(conn, m)?;
            crate::settings::set(conn, &member_key(m), p)
        }
    }
}

/// Instruções do agente para quem está usando agora: a família (quem administra) ou um
/// membro, com o nível e o tom dele. Em inglês, como o resto das instruções (o agente
/// responde no idioma da instalação).
pub fn prompt_line_for(conn: &Connection, member_id: Option<&str>) -> Result<String> {
    let p = get_for(conn, member_id)?;
    let line = prompt_line(&p);
    Ok(match member_id {
        None => line,
        Some(m) => {
            let name = crate::household::get_member(conn, m)?.name;
            format!(
                "Who is using the app now: {name}, a family member (account holder). Talk to \
                 {name} and, when it makes sense, focus on their own finances (transactions, \
                 accounts and debts they hold). {line}"
            )
        }
    })
}

/// O tom de quem está usando agora (o do membro, ou o da família), para o lembrete do fim das
/// instruções.
pub fn tone_now(conn: &Connection, member_id: Option<&str>) -> Option<Tone> {
    get_for(conn, member_id).ok().map(|p| p.tone)
}

/// Perfil a partir das respostas do questionário.
pub fn from_answers(answers: BTreeMap<String, Vec<String>>) -> Result<Profile> {
    validate(&answers)?;
    let (level, tone) = score(&answers);
    Ok(Profile {
        level,
        tone,
        source: Source::Questionario,
        goals: answers.get("objetivo").cloned().unwrap_or_default(),
        income: answers
            .get("renda")
            .and_then(|r| r.first())
            .filter(|r| r.as_str() != "nao_informar")
            .cloned(),
        answers,
        onboarding_pending: false,
        updated_at: Some(crate::time::now_utc()),
    })
}

/// Como falar com esta pessoa, para o agente (em inglês, como o resto das instruções). Diz o
/// nível que ela declarou no questionário, a linguagem escolhida e o idioma do app, com regras
/// concretas: o agente tende a responder em linguagem técnica se nada o segura.
pub fn prompt_line(p: &Profile) -> String {
    let lang = crate::ai::prompt::language_name(crate::i18n::current());
    let level = match p.level {
        Level::Iniciante => "a beginner in personal finance and investing",
        Level::Intermediario => "at an intermediate level in personal finance",
        Level::Avancado => "advanced in personal finance and investing",
    };
    let tone = match p.tone {
        Tone::Simples => {
            "very simple language: short sentences, everyday words, no jargon; explain any \
             unavoidable term (CDI, Selic, liquidity, volatility) in a few words the first time. \
             Lead with the answer and the next step, one or two rounded numbers and an everyday \
             example instead of tables and formulas; encouraging, never condescending. Keep it \
             simple even when the question or a tool result is technical, unless they ask for \
             detail"
        }
        Tone::Equilibrado => {
            "clear language: technical terms only when they help, with a short explanation of \
             the less common ones; no unnecessary jargon, never condescending"
        }
        Tone::Tecnico => {
            "technical language: precise terms, numbers and detailed analyses are welcome"
        }
    };
    let mut s = format!(
        "The person using the app is {level} (from their profile) and prefers {tone}. Write in \
         {lang}."
    );
    let goals: Vec<&str> = p
        .goals
        .iter()
        .filter_map(|g| match g.as_str() {
            "onde_vai" => Some("knowing where the money goes"),
            "reserva" => Some("building an emergency fund"),
            "dividas" => Some("getting out of debt"),
            "objetivo" => Some("saving for a goal"),
            "organizar" => Some("organizing the finances"),
            "investir" => Some("investing"),
            _ => None,
        })
        .collect();
    if !goals.is_empty() {
        s.push_str(&format!(" Goals: {}.", goals.join(", ")));
    }
    match p.income.as_deref() {
        Some("variavel") => {
            s.push_str(" Variable income: take weaker months into account in the tips.")
        }
        Some("beneficios") => s.push_str(
            " Income comes from government benefits: realistic tips, without judgment, starting \
             from the predictable monthly amount.",
        ),
        _ => {}
    }
    s
}
