//! Busca de ferramentas por intenção (`find_tools`): a pergunta do cliente ("quitar
//! empréstimo", "cheque especial", "rebalancear") contra o nome, o título e a descrição de cada
//! ferramenta, com sinônimos em português e espanhol que levam aos termos em inglês do
//! catálogo. Só entram as ferramentas que o cliente recebe ([`Access`]).

use serde::{Deserialize, Serialize};

use super::tools::{self, Access, ToolSpec};

/// Quantas voltam por padrão e no máximo.
pub const DEFAULT_LIMIT: usize = 5;
pub const MAX_LIMIT: usize = 15;

/// Palavras da pergunta (sem acento, em minúsculas) e os termos do catálogo que elas
/// indicam. Os termos em inglês da própria pergunta valem como estão.
const SYNONYMS: &[(&str, &[&str])] = &[
    ("quitar", &["payoff", "prepayment", "prepay"]),
    ("quitacao", &["payoff", "prepayment"]),
    ("amortizar", &["prepayment", "amortization"]),
    ("amortizacao", &["prepayment", "amortization"]),
    ("antecipar", &["prepayment", "prepay"]),
    ("emprestimo", &["loan", "debt"]),
    ("emprestimos", &["loan", "debt"]),
    ("prestamo", &["loan", "debt"]),
    ("financiamento", &["loan", "amortization"]),
    ("divida", &["debt", "loan"]),
    ("dividas", &["debt", "loan"]),
    ("deuda", &["debt", "loan"]),
    ("cheque", &["overdraft"]),
    ("especial", &["overdraft"]),
    ("descoberto", &["overdraft"]),
    ("juros", &["interest", "rate", "rates"]),
    ("taxa", &["rate", "rates", "fee"]),
    ("taxas", &["rate", "rates", "fee"]),
    ("rebalancear", &["rebalance"]),
    ("rebalanceamento", &["rebalance"]),
    ("carteira", &["portfolio", "position"]),
    ("investimento", &["portfolio", "investment", "trade"]),
    ("investimentos", &["portfolio", "investment", "trade"]),
    ("acao", &["asset", "stock"]),
    ("acoes", &["asset", "stock"]),
    ("ativo", &["asset"]),
    ("ativos", &["asset"]),
    ("compra", &["trade", "buy", "purchase"]),
    ("venda", &["trade", "sell", "sale", "sales"]),
    ("dividendo", &["dividend", "income"]),
    ("dividendos", &["dividend", "income"]),
    ("provento", &["dividend", "income"]),
    ("proventos", &["dividend", "income"]),
    ("renda", &["income", "passive"]),
    ("receita", &["income"]),
    ("salario", &["payslip", "income"]),
    ("holerite", &["payslip"]),
    ("gasto", &["spending", "expense"]),
    ("gastos", &["spending", "expense"]),
    ("despesa", &["spending", "expense"]),
    ("despesas", &["spending", "expense"]),
    ("fluxo", &["flow", "cash"]),
    ("caixa", &["cash", "flow"]),
    ("orcamento", &["budget"]),
    ("presupuesto", &["budget"]),
    ("saldo", &["balance"]),
    ("previsao", &["forecast"]),
    ("patrimonio", &["net", "worth"]),
    ("meta", &["goal"]),
    ("metas", &["goal"]),
    ("objetivo", &["goal"]),
    ("reserva", &["goal", "emergency"]),
    ("emergencia", &["emergency", "goal"]),
    ("aposentadoria", &["retirement"]),
    ("jubilacion", &["retirement"]),
    ("previdencia", &["pension"]),
    ("simular", &["simulate", "scenario"]),
    ("simulacao", &["simulate", "scenario"]),
    ("cenario", &["scenario"]),
    ("imposto", &["tax"]),
    ("impostos", &["tax"]),
    ("impuesto", &["tax"]),
    ("ir", &["tax"]),
    ("darf", &["tax"]),
    ("cartao", &["card"]),
    ("tarjeta", &["card"]),
    ("fatura", &["card", "statement"]),
    ("extrato", &["statement"]),
    ("conta", &["account"]),
    ("contas", &["account"]),
    ("cuenta", &["account"]),
    ("lancamento", &["transaction"]),
    ("lancamentos", &["transaction"]),
    ("transacao", &["transaction"]),
    ("transacoes", &["transaction"]),
    ("categoria", &["category", "categorize"]),
    ("categorizar", &["categorize", "category"]),
    ("transferencia", &["transfer"]),
    ("duplicado", &["duplicate"]),
    ("duplicados", &["duplicate"]),
    ("parcela", &["installment"]),
    ("parcelas", &["installment"]),
    ("assinatura", &["subscription"]),
    ("assinaturas", &["subscription"]),
    ("recorrente", &["recurring", "recurrence"]),
    ("cambio", &["exchange", "currency", "fx"]),
    ("dolar", &["currency", "fx", "exchange"]),
    ("moeda", &["currency"]),
    ("converter", &["convert", "currency"]),
    ("inflacao", &["inflation"]),
    ("cesta", &["basket"]),
    ("supermercado", &["basket", "purchases", "receipts"]),
    ("compras", &["purchases", "basket"]),
    ("indice", &["index", "series"]),
    ("cotacao", &["price", "history"]),
    ("preco", &["price"]),
    ("noticia", &["news"]),
    ("noticias", &["news"]),
    ("documento", &["document"]),
    ("pdf", &["document", "read"]),
    ("ler", &["read"]),
    ("leia", &["read"]),
    ("documentos", &["document"]),
    ("nota", &["note", "receipt"]),
    ("notas", &["note", "receipt"]),
    ("cupom", &["receipt"]),
    ("imovel", &["physical", "property"]),
    ("imoveis", &["physical", "property"]),
    ("familia", &["family", "member"]),
    ("membro", &["member"]),
    ("importar", &["import"]),
    ("proposta", &["proposal"]),
    ("propostas", &["proposal"]),
    ("alerta", &["alert"]),
    ("alertas", &["alert"]),
    ("saude", &["health", "score"]),
    ("risco", &["risk"]),
    ("comparar", &["compare"]),
    ("ranking", &["rank"]),
    ("projeto", &["project"]),
    ("projetos", &["project"]),
    ("historico", &["history"]),
    ("desfazer", &["history", "change"]),
];

/// Palavras que não dizem nada sozinhas.
const STOP: &[&str] = &[
    "a", "o", "as", "os", "de", "do", "da", "dos", "das", "e", "em", "no", "na", "um", "uma",
    "para", "por", "com", "meu", "minha", "the", "of", "to", "and", "for", "my", "in", "el", "la",
    "los", "las", "mi", "y", "en", "que", "como",
];

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct FindArgs {
    pub query: String,
    pub limit: Option<usize>,
    /// O que o cliente recebe (posto pelo servidor, nunca pelo cliente).
    pub access: Option<AccessArgs>,
}

/// [`Access`] em JSON (o comando da API recebe por argumento).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AccessArgs {
    pub writes: bool,
    pub agent: bool,
    pub dev: bool,
    pub longevity: bool,
    pub outside_brazil: bool,
}

impl From<Access> for AccessArgs {
    fn from(a: Access) -> Self {
        Self {
            writes: a.writes,
            agent: a.agent,
            dev: a.dev,
            longevity: a.longevity,
            outside_brazil: a.outside_brazil,
        }
    }
}

impl From<AccessArgs> for Access {
    fn from(a: AccessArgs) -> Self {
        Self {
            writes: a.writes,
            agent: a.agent,
            dev: a.dev,
            longevity: a.longevity,
            outside_brazil: a.outside_brazil,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Found {
    pub name: &'static str,
    pub title: &'static str,
    /// A primeira frase da descrição: o que a ferramenta faz.
    pub usage: String,
    /// `true`: só lê; `false`: grava (em geral, uma proposta para a pessoa aprovar).
    pub read_only: bool,
}

/// Sem acento, em minúsculas, em palavras.
fn words(s: &str) -> Vec<String> {
    let folded: String = s
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' | 'Á' | 'À' | 'Â' | 'Ã' => 'a',
            'é' | 'ê' | 'è' | 'É' | 'Ê' => 'e',
            'í' | 'Í' => 'i',
            'ó' | 'ô' | 'õ' | 'ö' | 'Ó' | 'Ô' | 'Õ' => 'o',
            'ú' | 'ü' | 'Ú' => 'u',
            'ç' | 'Ç' => 'c',
            'ñ' | 'Ñ' => 'n',
            c => c.to_ascii_lowercase(),
        })
        .collect();
    folded
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect()
}

/// Termos procurados: as palavras da pergunta e o que os sinônimos indicam.
fn terms(query: &str) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    for w in words(query) {
        if STOP.contains(&w.as_str()) {
            continue;
        }
        for (k, v) in SYNONYMS {
            if *k == w {
                out.extend(v.iter().map(|s| s.to_string()));
            }
        }
        out.push(w);
    }
    out.sort();
    out.dedup();
    out
}

/// Um termo bate com uma palavra igual ou que comece por ele (`loan` × `loans`).
fn hits(term: &str, list: &[String]) -> bool {
    list.iter()
        .any(|w| w == term || (term.len() >= 4 && w.starts_with(term)))
}

fn score(spec: &ToolSpec, terms: &[String]) -> u32 {
    let name = words(spec.name);
    let title = words(spec.title);
    let desc = words(spec.description);
    terms
        .iter()
        .map(|t| {
            if hits(t, &name) {
                3
            } else if hits(t, &title) {
                2
            } else if hits(t, &desc) {
                1
            } else {
                0
            }
        })
        .sum()
}

/// Quantas palavras da pergunta (cada uma com os seus sinônimos) a ferramenta cobre: no
/// empate, quem responde à pergunta inteira vem antes ("rebalancear a carteira").
fn coverage(spec: &ToolSpec, query: &str) -> usize {
    let all: Vec<String> = [spec.name, spec.title, spec.description]
        .iter()
        .flat_map(|s| words(s))
        .collect();
    words(query)
        .into_iter()
        .filter(|w| !STOP.contains(&w.as_str()))
        .filter(|w| terms(w).iter().any(|t| hits(t, &all)))
        .count()
}

fn usage(description: &str) -> String {
    let one: String = description.split_whitespace().collect::<Vec<_>>().join(" ");
    match one.find(". ") {
        Some(i) => one[..=i].to_string(),
        None => one,
    }
}

/// As ferramentas que melhor respondem à pergunta, da mais para a menos provável.
pub fn search(query: &str, access: Access, limit: usize) -> Vec<Found> {
    let terms = terms(query);
    if terms.is_empty() {
        return vec![];
    }
    let mut scored: Vec<(u32, &ToolSpec)> = tools::visible(access)
        .filter(|t| t.name != "find_tools")
        .map(|t| (score(t, &terms), t))
        .filter(|(s, _)| *s > 0)
        .collect();
    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| coverage(b.1, query).cmp(&coverage(a.1, query)))
            .then(a.1.name.cmp(b.1.name))
    });
    scored
        .into_iter()
        .take(limit.clamp(1, MAX_LIMIT))
        .map(|(_, t)| Found {
            name: t.name,
            title: t.title,
            usage: usage(t.description),
            read_only: t.is_read_only(),
        })
        .collect()
}

/// Ferramentas que já existem para o que um feedback novo pede (título e texto): só as de
/// pontuação alta (`min_score`), para não sugerir qualquer coisa. Vê todas as ferramentas de
/// escrita e leitura, menos as do próprio feedback.
pub fn suggest(text: &str, min_score: u32, limit: usize) -> Vec<Found> {
    let terms = terms(text);
    if terms.is_empty() {
        return vec![];
    }
    let access = Access {
        writes: true,
        ..Default::default()
    };
    let mut scored: Vec<(u32, &ToolSpec)> = tools::visible(access)
        .filter(|t| t.name != "find_tools" && !t.command.starts_with("devfeedback."))
        .map(|t| (score(t, &terms), t))
        .filter(|(s, _)| *s >= min_score)
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.name.cmp(b.1.name)));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, t)| Found {
            name: t.name,
            title: t.title,
            usage: usage(t.description),
            read_only: t.is_read_only(),
        })
        .collect()
}

/// `ai.find_tools`.
pub fn run(args: FindArgs) -> Vec<Found> {
    let access = args.access.map(Access::from).unwrap_or_default();
    search(&args.query, access, args.limit.unwrap_or(DEFAULT_LIMIT))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(q: &str) -> Vec<&'static str> {
        let all = Access {
            writes: true,
            ..Access::default()
        };
        search(q, all, 5).into_iter().map(|f| f.name).collect()
    }

    #[test]
    fn finds_by_intent_in_portuguese() {
        let loan = names("quitar empréstimo");
        assert!(
            loan[..2].contains(&"simulate_loan_prepayment")
                && loan[..2].contains(&"plan_debt_payoff"),
            "{loan:?}"
        );
        let reb = names("rebalancear a carteira");
        assert!(reb.contains(&"rebalance_suggestion"), "{reb:?}");
        assert!(
            names("cheque especial")
                .iter()
                .any(|n| n.contains("loan") || n.contains("debt"))
        );
        assert!(names("fluxo de caixa do mês").contains(&"get_monthly_flow"));
        assert!(names("").is_empty());
        assert!(names("de para").is_empty());
    }

    #[test]
    fn only_the_tools_the_client_gets() {
        let read = Access::default();
        let found = search("registrar compra de ação", read, 15);
        assert!(found.iter().all(|f| f.read_only), "{found:?}");
        assert!(found.iter().all(|f| f.name != "record_trade"));
        assert!(
            search(
                "registrar compra de ação",
                Access {
                    writes: true,
                    ..read
                },
                15
            )
            .iter()
            .any(|f| f.name == "record_trade")
        );
        let usage = &search("quitar empréstimo", read, 1)[0].usage;
        assert!(usage.ends_with('.') && !usage.is_empty(), "{usage}");
    }
}
