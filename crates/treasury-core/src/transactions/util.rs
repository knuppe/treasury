//! Utilitários internos do livro-caixa (contas, categorias, lançamentos, cartões,
//! recorrências e família): limpeza de texto, listas de parâmetros SQL e checagens de
//! existência com mensagens no idioma da instalação (`nucleo-lancamentos.ftl`).

use rusqlite::{Connection, OptionalExtension};

use crate::error::{Error, Result};
use crate::i18n::{args, t_args};

/// Tamanho máximo de nomes curtos (contas, categorias, tags, membros).
pub(crate) const MAX_NAME: usize = 120;

/// `Some("  x ")` → `Some("x")`; texto vazio vira `None`.
pub(crate) fn clean_opt(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// De quem é o nome em [`clean_name`]: o `what` em pt-BR ("da conta") é o identificador
/// interno; a mensagem sai do catálogo, no idioma da instalação.
fn name_subject(what: &str) -> &'static str {
    match what {
        "da conta" => "conta",
        "da categoria" => "categoria",
        "da tag" => "tag",
        "do membro" => "membro",
        "da família" => "familia",
        "da instituição" => "instituicao",
        "da renda" => "renda",
        _ => "outro",
    }
}

/// Nome obrigatório, sem espaços nas pontas e com tamanho limitado.
pub(crate) fn clean_name(s: &str, what: &str) -> Result<String> {
    let v = s.trim();
    let subject = name_subject(what);
    if v.is_empty() {
        return Err(Error::invalid(t_args(
            "nucleo-lancamentos-erro-nome-vazio",
            &args(&[("coisa", subject.into())]),
        )));
    }
    if v.chars().count() > MAX_NAME {
        return Err(Error::invalid(t_args(
            "nucleo-lancamentos-erro-nome-longo",
            &args(&[("coisa", subject.into()), ("max", MAX_NAME.into())]),
        )));
    }
    Ok(v.to_string())
}

/// Cor no formato `#rgb` ou `#rrggbb` (ou nome de token da interface, sem espaços).
pub(crate) fn clean_color(s: Option<String>) -> Result<Option<String>> {
    let Some(c) = clean_opt(s) else {
        return Ok(None);
    };
    if c.len() > 32 || c.chars().any(char::is_whitespace) {
        return Err(Error::invalid(t_args(
            "nucleo-lancamentos-erro-cor",
            &args(&[("cor", c.into())]),
        )));
    }
    Ok(Some(c))
}

/// `?,?,?` com `n` marcadores.
pub(crate) fn placeholders(n: usize) -> String {
    vec!["?"; n].join(",")
}

/// A linha existe na tabela? `table` é sempre um nome fixo do código.
pub(crate) fn exists(conn: &Connection, table: &str, id: &str) -> Result<bool> {
    Ok(conn
        .query_row(
            &format!("SELECT 1 FROM {table} WHERE id = ?1"),
            [id],
            |_| Ok(()),
        )
        .optional()?
        .is_some())
}

pub(crate) fn require_member(conn: &Connection, id: &str) -> Result<()> {
    if !exists(conn, "member", id)? {
        return Err(Error::not_found(member_ref(id)));
    }
    Ok(())
}

/// "membro {id}" (detalhe do erro "não encontrado").
pub(crate) fn member_ref(id: &str) -> String {
    t_args(
        "nucleo-lancamentos-ref-membro",
        &args(&[("id", id.to_string().into())]),
    )
}

/// "lançamento {id}" (detalhe do erro "não encontrado").
pub(crate) fn txn_ref(id: &str) -> String {
    t_args(
        "nucleo-lancamentos-ref-lancamento",
        &args(&[("id", id.to_string().into())]),
    )
}

/// Ids distintos preservando a ordem de chegada.
pub(crate) fn dedup_ids(ids: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    ids.iter()
        .filter(|id| seen.insert(id.as_str()))
        .cloned()
        .collect()
}

/// Data para mensagens e motivos, no idioma da instalação (`2026-10-14` → `14/10/2026`,
/// `Oct 14, 2026`).
pub(crate) fn date_text(d: &str) -> String {
    match crate::time::parse_date(d) {
        Ok(x) => crate::i18n::date(crate::i18n::current(), x),
        Err(_) => d.to_string(),
    }
}

/// Dinheiro para mensagens, no idioma da instalação; moedas sem duas casas decimais (como o
/// iene) ficam no formato de sempre, que respeita as casas de cada moeda.
pub(crate) fn money_text(amount: i64, currency: &str) -> String {
    if crate::money::minor_units(currency) == 2 {
        crate::i18n::money(crate::i18n::current(), amount, currency)
    } else {
        crate::money::format_brl_style(amount, currency)
    }
}

/// Divide `a / b` arredondando metade para longe do zero (inteiros, sem ponto flutuante).
pub(crate) fn div_round(a: i128, b: i128) -> i128 {
    debug_assert!(b != 0);
    let (a, b) = if b < 0 { (-a, -b) } else { (a, b) };
    let q = a / b;
    let r = a % b;
    if 2 * r.abs() >= b { q + a.signum() } else { q }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounding_and_cleaning() {
        assert_eq!(div_round(10, 4), 3);
        assert_eq!(div_round(-10, 4), -3);
        assert_eq!(div_round(9, 4), 2);
        assert_eq!(div_round(-9, 4), -2);
        assert_eq!(div_round(7, 7), 1);
        assert_eq!(clean_opt(Some("  ".into())), None);
        assert_eq!(clean_opt(Some(" a ".into())), Some("a".into()));
        assert!(clean_name("   ", "da conta").is_err());
        assert_eq!(date_text("2026-10-14"), "14/10/2026");
        assert_eq!(
            dedup_ids(&["a".into(), "b".into(), "a".into()]),
            vec!["a".to_string(), "b".to_string()]
        );
    }
}
