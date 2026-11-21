//! Termos de uso (docs/terms.md): a versão vigente e o aceite desta instalação. O aceite fica
//! em `terms-accepted.json`, na pasta do app e fora do cofre (a tela aparece antes de existir
//! um cofre), e nada é enviado. Versão própria MAIOR.MENOR, independente do app: mudar o MAIOR
//! pede um novo aceite na abertura; mudar o MENOR só registra (aviso discreto, sem bloquear).

use std::path::Path;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};

/// Versão vigente dos termos.
pub const CURRENT: &str = "1.0";
const FILE: &str = "terms-accepted.json";

/// O que mudou em cada versão (a mais nova primeiro), para o resumo no topo da tela: a versão
/// e a chave do resumo no catálogo (`nucleo-geral.ftl`, no idioma da instalação).
pub const HISTORY: &[(&str, &str)] = &[("1.0", "nucleo-geral-termos-1-0")];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TermsAcceptance {
    pub version: String,
    /// Instante (UTC, RFC 3339).
    pub accepted_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct TermsVersion {
    pub version: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct TermsStatus {
    pub current: String,
    pub accepted: Option<TermsAcceptance>,
    /// Mostrar a tela e pedir o aceite (nunca aceitou, ou o MAIOR aceito é menor que o atual).
    pub needs_acceptance: bool,
    /// O MENOR mudou: aviso discreto "Os termos foram atualizados", sem bloquear.
    pub updated: bool,
    /// Versões depois da aceita (o "O que mudou"), a mais nova primeiro.
    pub changes: Vec<TermsVersion>,
    /// Todas as versões (o histórico em Ajustes), a mais nova primeiro.
    pub history: Vec<TermsVersion>,
}

fn parse(v: &str) -> Option<(u32, u32)> {
    let (a, b) = v.trim().split_once('.')?;
    Some((a.parse().ok()?, b.parse().ok()?))
}

/// Precisa aceitar? E houve mudança só de MENOR? (função pura)
pub fn evaluate(accepted: Option<&str>, current: &str) -> (bool, bool) {
    let Some(cur) = parse(current) else {
        return (true, false);
    };
    match accepted.and_then(parse) {
        None => (true, false),
        Some(acc) if acc.0 < cur.0 => (true, false),
        // Aceite de uma versão mais nova que a vigente (os termos recomeçaram em 1.0 na
        // publicação, ou o app voltou de versão): o texto aceito não é este, pede de novo.
        Some(acc) if acc > cur => (true, false),
        Some(acc) => (false, acc < cur),
    }
}

pub fn load(root: &Path) -> Option<TermsAcceptance> {
    std::fs::read_to_string(root.join(FILE))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

pub fn status(root: &Path) -> TermsStatus {
    let accepted = load(root);
    let (needs_acceptance, updated) =
        evaluate(accepted.as_ref().map(|a| a.version.as_str()), CURRENT);
    let after = accepted.as_ref().and_then(|a| parse(&a.version));
    let changes = HISTORY
        .iter()
        .filter(|(v, _)| matches!((after, parse(v)), (Some(a), Some(x)) if x > a))
        .map(|(v, s)| TermsVersion {
            version: (*v).into(),
            summary: t(s),
        })
        .collect();
    TermsStatus {
        current: CURRENT.into(),
        accepted,
        needs_acceptance,
        updated,
        changes,
        history: HISTORY
            .iter()
            .map(|(v, s)| TermsVersion {
                version: (*v).into(),
                summary: t(s),
            })
            .collect(),
    }
}

/// Grava o aceite da versão vigente (só a interface, com a caixa marcada pelo usuário).
pub fn accept(root: &Path, version: &str) -> Result<TermsStatus> {
    if version != CURRENT {
        return Err(Error::conflict(t_args(
            "nucleo-geral-erro-termos-mudaram",
            &args(&[("versao", CURRENT.into())]),
        )));
    }
    crate::paths::create_private_dir(root)?;
    let a = TermsAcceptance {
        version: CURRENT.into(),
        accepted_at: crate::time::now_utc(),
    };
    let p = root.join(FILE);
    let tmp = p.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(&a)?)?;
    crate::paths::set_mode(&tmp, 0o600)?;
    std::fs::rename(&tmp, &p)?;
    Ok(status(root))
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct TermsAcceptArgs {
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_decide_when_to_ask() {
        assert_eq!(evaluate(None, "1.0"), (true, false));
        assert_eq!(evaluate(Some("1.0"), "1.0"), (false, false));
        assert_eq!(
            evaluate(Some("1.0"), "1.1"),
            (false, true),
            "menor: só aviso"
        );
        assert_eq!(
            evaluate(Some("1.1"), "2.0"),
            (true, false),
            "maior: novo aceite"
        );
        assert_eq!(evaluate(Some("lixo"), "1.0"), (true, false));
        assert_eq!(
            evaluate(Some("1.9"), "1.10"),
            (false, true),
            "1.10 é depois de 1.9"
        );
        assert_eq!(
            evaluate(Some("2.2"), "1.0"),
            (true, false),
            "aceite de uma versão mais nova: pede de novo"
        );
    }

    /// docs/terms.pt-BR.md é a fonte; o app usa a mesma cópia do texto em pt-BR, e o
    /// docs/terms.md (em inglês) traz a cópia em inglês que o app mostra.
    #[test]
    fn app_copy_matches_docs() {
        for (doc, app, marker) in [
            (
                include_str!("../../../docs/terms.pt-BR.md"),
                include_str!("../../../locales/conteudo/pt-BR/termos.md"),
                "<!-- texto completo: igual a locales/conteudo/pt-BR/termos.md -->",
            ),
            (
                include_str!("../../../docs/terms.md"),
                include_str!("../../../locales/conteudo/en/termos.md"),
                "<!-- full text: same as locales/conteudo/en/termos.md -->",
            ),
        ] {
            let (_, full) = doc
                .split_once(marker)
                .expect("marcador no documento dos termos");
            assert_eq!(full.trim(), app.trim());
        }
    }

    #[test]
    fn acceptance_is_local_and_versioned() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".treasury");
        assert!(status(&root).needs_acceptance);
        assert!(accept(&root, "0.9").is_err());
        let s = accept(&root, CURRENT).unwrap();
        assert!(!s.needs_acceptance && !s.updated);
        assert_eq!(s.accepted.unwrap().version, CURRENT);
        assert!(root.join(FILE).exists());
    }
}
