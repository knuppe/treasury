//! Prompts MCP: "Revisão mensal" e "Plano para meta". Orientam o modelo a usar as
//! ferramentas do Treasury e a citar os números e a origem de cada um. O pedido vai como
//! mensagem do usuário, no idioma da instalação (`locales/<idioma>/mcp.ftl`); as descrições
//! ficam em inglês, como as das ferramentas.

use rmcp::model::{GetPromptResult, JsonObject, Prompt, PromptArgument, PromptMessage, Role};
use serde_json::Value;

use crate::i18n::{self, t, t_args};

pub(crate) const MONTHLY_REVIEW: &str = "revisao_mensal";
pub(crate) const GOAL_PLAN: &str = "plano_para_meta";

/// Títulos no idioma da instalação (o cliente mostra ao usuário); descrições em inglês.
pub(crate) fn list() -> Vec<Prompt> {
    vec![
        Prompt::new(
            MONTHLY_REVIEW,
            Some(
                "Monthly close: income vs. expenses, budget, largest expenses, card statements, \
                 goals and 3 to 5 concrete recommendations, citing the numbers and the tools.",
            ),
            Some(vec![
                PromptArgument::new("mes")
                    .with_title(t("mcp-prompt-revisao-arg-mes"))
                    .with_description("Month as YYYY-MM (default: current month).")
                    .with_required(false),
            ]),
        )
        .with_title(t("mcp-prompt-revisao-titulo")),
        Prompt::new(
            GOAL_PLAN,
            Some(
                "Plan to reach a goal (emergency fund, trip, paying off a debt…) based on the \
                 family's real cash flow: monthly contribution, deadline and where to save.",
            ),
            Some(vec![
                PromptArgument::new("meta")
                    .with_title(t("mcp-prompt-meta-arg-meta"))
                    .with_description(
                        "Name or id of a registered goal, or the description of a new goal (e.g. \
                         \"a 15,000 trip in December 2027\").",
                    )
                    .with_required(false),
            ]),
        )
        .with_title(t("mcp-prompt-meta-titulo")),
    ]
}

fn arg(args: &Option<JsonObject>, name: &str) -> Option<String> {
    args.as_ref()?
        .get(name)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().take(200).collect())
}

/// `base`: a moeda base da família (os exemplos de dinheiro das regras); sem cofre, BRL.
pub(crate) fn get(
    name: &str,
    args: &Option<JsonObject>,
    base: Option<&str>,
) -> Option<Result<GetPromptResult, String>> {
    let base = base.unwrap_or("BRL");
    match name {
        MONTHLY_REVIEW => Some(monthly_review(arg(args, "mes"), base)),
        GOAL_PLAN => Some(Ok(goal_plan(arg(args, "meta"), base))),
        _ => None,
    }
}

/// Pedido do prompt (no idioma da instalação) com as regras comuns no fim.
fn with_rules(body: String, base: &str) -> String {
    let l = i18n::current();
    let rules = t_args(
        "mcp-prompt-regras",
        &i18n::args(&[
            ("moeda", base.to_string().into()),
            ("exemplo", i18n::money(l, 123_456, base).into()),
            ("despesas", i18n::money(l, 842_010, base).into()),
        ]),
    );
    format!("{body}\n\n{rules}")
}

fn monthly_review(month: Option<String>, base: &str) -> Result<GetPromptResult, String> {
    let month = match month {
        Some(m) => {
            let (y, mo) = crate::time::parse_month(&m).map_err(|e| e.to_string())?;
            crate::time::fmt_month(y, mo)
        }
        None => crate::time::month_of(crate::time::today_local()),
    };
    let a = i18n::args(&[("mes", month.into())]);
    let text = with_rules(t_args("mcp-prompt-revisao-texto", &a), base);
    Ok(
        GetPromptResult::new(vec![PromptMessage::new_text(Role::User, text)])
            .with_description(t_args("mcp-prompt-revisao-descricao", &a)),
    )
}

fn goal_plan(goal: Option<String>, base: &str) -> GetPromptResult {
    let target = match &goal {
        Some(g) => t_args(
            "mcp-prompt-meta-alvo",
            &i18n::args(&[("meta", g.clone().into())]),
        ),
        None => t("mcp-prompt-meta-alvo-livre"),
    };
    let text = with_rules(
        t_args(
            "mcp-prompt-meta-texto",
            &i18n::args(&[("alvo", target.into())]),
        ),
        base,
    );
    GetPromptResult::new(vec![PromptMessage::new_text(Role::User, text)]).with_description(
        match goal {
            Some(g) => t_args(
                "mcp-prompt-meta-descricao",
                &i18n::args(&[("meta", g.into())]),
            ),
            None => t("mcp-prompt-meta-titulo"),
        },
    )
}
