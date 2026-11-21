//! Partes de uma resposta do agente: o raciocínio do modelo e os textos intermediários
//! ficam recolhidos, as ferramentas viram uma lista compacta e o texto depois da última
//! ferramenta é a resposta.
//!
//! Regra única, igual à do streaming na interface (`chatStream.ts`), para que a conversa
//! gravada fique como o usuário viu ao vivo:
//!
//! - eventos `reasoning` são raciocínio;
//! - texto seguido de uma chamada de ferramenta é um passo intermediário ("Vou consultar o
//!   orçamento…") e entra no raciocínio;
//! - o texto que sobra depois da última ferramenta é a resposta; se não sobrar nada, o
//!   último passo intermediário vira a resposta;
//! - uma troca de motor (fallback) descarta o texto parcial do motor que falhou.
//!
//! Vale para todos os motores: API, modelo local e CLI (cuja saída também vira eventos).

use chrono::DateTime;

use super::conversation::ToolCallBrief;
use super::jobs::{JobEvent, JobEventKind};
use crate::ids::Id;

/// Teto do raciocínio gravado com a mensagem (o resto é cortado).
pub const MAX_REASONING: usize = 20_000;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReplyParts {
    /// Resposta final.
    pub answer: String,
    /// Raciocínio e passos intermediários, na ordem, separados por linha em branco.
    pub reasoning: String,
    /// Tempo do começo da resposta até o texto final começar, quando houve raciocínio ou
    /// ferramentas.
    pub reasoning_ms: Option<i64>,
    pub tools: Vec<ToolCallBrief>,
    pub proposals: Vec<Id>,
}

#[derive(Debug, Clone, PartialEq)]
enum Step {
    Reasoning(String),
    Narration(String),
}

fn millis(at: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(at)
        .ok()
        .map(|d| d.timestamp_millis())
}

fn cut(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    format!("{}…", s.chars().take(max - 1).collect::<String>())
}

/// Separa os eventos de um job de chat em resposta, raciocínio e ferramentas.
pub fn split(events: &[JobEvent]) -> ReplyParts {
    let mut steps: Vec<Step> = vec![];
    let mut text = String::new();
    let mut reasoning = String::new();
    let mut tools: Vec<ToolCallBrief> = vec![];
    let mut proposals: Vec<Id> = vec![];
    let mut first_at: Option<i64> = None;
    // Começo do texto em andamento (o da resposta, se nenhuma ferramenta vier depois).
    let mut text_at: Option<i64> = None;
    let mut last_at: Option<i64> = None;

    let flush_reasoning = |reasoning: &mut String, steps: &mut Vec<Step>| {
        let r = reasoning.trim();
        if !r.is_empty() {
            steps.push(Step::Reasoning(r.to_string()));
        }
        reasoning.clear();
    };

    for e in events {
        let at = millis(&e.at);
        if first_at.is_none() {
            first_at = at;
        }
        if !matches!(e.event, JobEventKind::Done { .. }) {
            last_at = at.or(last_at);
        }
        if !matches!(e.event, JobEventKind::Reasoning { .. }) {
            flush_reasoning(&mut reasoning, &mut steps);
        }
        match &e.event {
            JobEventKind::Reasoning { delta } => reasoning.push_str(delta),
            JobEventKind::Text { delta } => {
                if text.trim().is_empty() && !delta.trim().is_empty() {
                    text_at = at;
                }
                text.push_str(delta);
            }
            JobEventKind::ToolCall { name, summary } => {
                let n = text.trim();
                if !n.is_empty() {
                    steps.push(Step::Narration(n.to_string()));
                }
                text.clear();
                text_at = None;
                tools.push(ToolCallBrief {
                    name: name.clone(),
                    args: summary.clone(),
                    summary: String::new(),
                    ok: false,
                });
            }
            JobEventKind::ToolResult { name, ok, summary } => {
                match tools
                    .iter_mut()
                    .rev()
                    .find(|t| &t.name == name && t.summary.is_empty() && !t.ok)
                {
                    Some(t) => {
                        t.ok = *ok;
                        t.summary = summary.clone();
                    }
                    None => tools.push(ToolCallBrief {
                        name: name.clone(),
                        args: String::new(),
                        summary: summary.clone(),
                        ok: *ok,
                    }),
                }
            }
            JobEventKind::Proposal { proposal_id, .. } => proposals.push(proposal_id.clone()),
            JobEventKind::Fallback { .. } => {
                text.clear();
                text_at = None;
            }
            _ => {}
        }
    }
    flush_reasoning(&mut reasoning, &mut steps);

    let mut answer = text.trim().to_string();
    if answer.is_empty()
        && let Some(i) = steps.iter().rposition(|s| matches!(s, Step::Narration(_)))
        && let Step::Narration(n) = steps.remove(i)
    {
        answer = n;
        text_at = None;
    }
    let reasoning_text = steps
        .iter()
        .map(|s| match s {
            Step::Reasoning(t) | Step::Narration(t) => t.as_str(),
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let busy = !reasoning_text.is_empty() || !tools.is_empty();
    let reasoning_ms = match (busy, first_at, text_at.or(last_at)) {
        (true, Some(a), Some(b)) => Some((b - a).max(0)),
        _ => None,
    };
    ReplyParts {
        answer,
        reasoning: cut(&reasoning_text, MAX_REASONING),
        reasoning_ms,
        tools,
        proposals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(seq: u64, secs: u32, event: JobEventKind) -> JobEvent {
        JobEvent {
            seq,
            at: format!("2026-10-15T12:00:{secs:02}.000Z"),
            event,
        }
    }

    fn text(t: &str) -> JobEventKind {
        JobEventKind::Text { delta: t.into() }
    }

    fn call(name: &str) -> JobEventKind {
        JobEventKind::ToolCall {
            name: name.into(),
            summary: "{\"month\":\"2026-10\"}".into(),
        }
    }

    fn result(name: &str, ok: bool) -> JobEventKind {
        JobEventKind::ToolResult {
            name: name.into(),
            ok,
            summary: if ok { "3 contas." } else { "Não encontrado." }.into(),
        }
    }

    #[test]
    fn narration_before_tools_goes_to_reasoning_and_answer_is_the_last_text() {
        let events = vec![
            ev(
                1,
                0,
                JobEventKind::Reasoning {
                    delta: "Preciso das ".into(),
                },
            ),
            ev(
                2,
                1,
                JobEventKind::Reasoning {
                    delta: "contas.".into(),
                },
            ),
            ev(3, 2, text("Vou consultar ")),
            ev(4, 2, text("as contas.")),
            ev(5, 3, call("list_accounts")),
            ev(6, 4, result("list_accounts", true)),
            ev(7, 5, text("\n\n")),
            ev(8, 6, call("get_account_balance")),
            ev(9, 7, result("get_account_balance", false)),
            ev(10, 12, text("Vocês têm ")),
            ev(11, 13, text("3 contas.")),
            ev(
                12,
                14,
                JobEventKind::Proposal {
                    proposal_id: "p1".into(),
                    tool: "create_rule".into(),
                    command: "rules.create".into(),
                },
            ),
        ];
        let p = split(&events);
        assert_eq!(p.answer, "Vocês têm 3 contas.");
        assert_eq!(
            p.reasoning,
            "Preciso das contas.\n\nVou consultar as contas."
        );
        assert_eq!(p.reasoning_ms, Some(12_000));
        assert_eq!(p.tools.len(), 2);
        assert_eq!(p.tools[0].args, "{\"month\":\"2026-10\"}");
        assert!(p.tools[0].ok && p.tools[0].summary == "3 contas.");
        assert!(!p.tools[1].ok && p.tools[1].summary == "Não encontrado.");
        assert_eq!(p.proposals, vec!["p1".to_string()]);
    }

    #[test]
    fn plain_answer_has_no_activity_and_engines_without_reasoning_still_work() {
        let p = split(&[ev(1, 0, text("Oi")), ev(2, 1, text("!"))]);
        assert_eq!(p.answer, "Oi!");
        assert_eq!(p.reasoning, "");
        assert_eq!(p.reasoning_ms, None);
        assert!(p.tools.is_empty());
        assert_eq!(split(&[]), ReplyParts::default());
    }

    #[test]
    fn last_narration_becomes_answer_when_nothing_follows_the_tools() {
        let p = split(&[
            ev(1, 0, text("Primeiro passo.")),
            ev(2, 1, call("a")),
            ev(3, 2, text("Anotei o caminho.")),
            ev(4, 3, call("remember_procedure")),
            ev(5, 4, result("remember_procedure", true)),
        ]);
        assert_eq!(p.answer, "Anotei o caminho.");
        assert_eq!(p.reasoning, "Primeiro passo.");
        assert_eq!(p.reasoning_ms, Some(4_000));
    }

    #[test]
    fn fallback_discards_partial_text_of_the_failed_engine() {
        let p = split(&[
            ev(1, 0, text("Resposta pela metade")),
            ev(
                2,
                1,
                JobEventKind::Fallback {
                    engine_id: "e1".into(),
                    engine_name: "A".into(),
                    reason: "rede".into(),
                },
            ),
            ev(3, 2, text("Resposta completa.")),
        ]);
        assert_eq!(p.answer, "Resposta completa.");
        assert_eq!(p.reasoning, "");
    }

    #[test]
    fn reasoning_is_capped() {
        let long = "x".repeat(MAX_REASONING + 50);
        let p = split(&[
            ev(1, 0, JobEventKind::Reasoning { delta: long }),
            ev(2, 1, text("ok")),
        ]);
        assert_eq!(p.reasoning.chars().count(), MAX_REASONING);
        assert!(p.reasoning.ends_with('…'));
    }
}
