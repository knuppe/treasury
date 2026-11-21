//! Recursos MCP: `treasury://summary/monthly/{yyyy-mm}` (painel do mês) e
//! `treasury://budget/{yyyy-mm}` (orçamento do mês), em JSON.

use rmcp::model::{Resource, ResourceTemplate};
use serde_json::{Value, json};

use crate::i18n::{args, t, t_args};

pub(crate) const SUMMARY_PREFIX: &str = "treasury://summary/monthly/";
pub(crate) const BUDGET_PREFIX: &str = "treasury://budget/";
pub(crate) const MIME_JSON: &str = "application/json";

/// Recurso pedido, já validado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResourceRef {
    MonthlySummary(String),
    Budget(String),
}

impl ResourceRef {
    pub fn parse(uri: &str) -> Option<Result<Self, String>> {
        let (month, make): (&str, fn(String) -> Self) = match uri.strip_prefix(SUMMARY_PREFIX) {
            Some(m) => (m, ResourceRef::MonthlySummary),
            None => (uri.strip_prefix(BUDGET_PREFIX)?, ResourceRef::Budget),
        };
        Some(match crate::time::parse_month(month) {
            Ok((y, m)) => Ok(make(crate::time::fmt_month(y, m))),
            Err(_) => Err(t_args(
                "mcp-recurso-mes-invalido",
                &args(&[("uri", uri.to_string().into())]),
            )),
        })
    }

    /// Comando da API e argumentos.
    pub fn command(&self) -> (&'static str, Value) {
        match self {
            ResourceRef::MonthlySummary(m) => ("reports.dashboard", json!({ "month": m })),
            ResourceRef::Budget(m) => ("budget.get", json!({ "month": m })),
        }
    }
}

/// Títulos no idioma da instalação (o cliente mostra ao usuário); descrições em inglês.
pub(crate) fn templates() -> Vec<ResourceTemplate> {
    vec![
        ResourceTemplate::new(format!("{SUMMARY_PREFIX}{{yyyy-mm}}"), "resumo_mensal")
            .with_title(t("mcp-recurso-resumo-titulo"))
            .with_description(
                "The month's dashboard (net worth, income vs. expenses, savings rate, largest \
                 categories, budget and upcoming due dates) as JSON. Amounts in cents.",
            )
            .with_mime_type(MIME_JSON),
        ResourceTemplate::new(format!("{BUDGET_PREFIX}{{yyyy-mm}}"), "orcamento_mensal")
            .with_title(t("mcp-recurso-orcamento-titulo"))
            .with_description(
                "The month's budget, planned vs. actual by category, as JSON. Amounts in cents.",
            )
            .with_mime_type(MIME_JSON),
    ]
}

/// Recursos concretos do mês atual (para clientes que não usam modelos de URI).
pub(crate) fn current(month: &str) -> Vec<Resource> {
    let mes = || args(&[("mes", month.to_string().into())]);
    vec![
        Resource::new(
            format!("{SUMMARY_PREFIX}{month}"),
            format!("resumo_{month}"),
        )
        .with_title(t_args("mcp-recurso-resumo-do-mes", &mes()))
        .with_description("The current month's dashboard as JSON. Amounts in cents.")
        .with_mime_type(MIME_JSON),
        Resource::new(
            format!("{BUDGET_PREFIX}{month}"),
            format!("orcamento_{month}"),
        )
        .with_title(t_args("mcp-recurso-orcamento-do-mes", &mes()))
        .with_description("The current month's budget as JSON. Amounts in cents.")
        .with_mime_type(MIME_JSON),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_resource_uris() {
        assert_eq!(
            ResourceRef::parse("treasury://summary/monthly/2026-09"),
            Some(Ok(ResourceRef::MonthlySummary("2026-09".into())))
        );
        assert_eq!(
            ResourceRef::parse("treasury://budget/2026-10"),
            Some(Ok(ResourceRef::Budget("2026-10".into())))
        );
        assert!(matches!(
            ResourceRef::parse("treasury://budget/2026-13"),
            Some(Err(_))
        ));
        assert_eq!(ResourceRef::parse("treasury://portfolio"), None);
        assert_eq!(
            ResourceRef::Budget("2026-10".into()).command(),
            ("budget.get", json!({"month": "2026-10"}))
        );
    }
}
