//! Arquivos da caixa de entrada nas ferramentas de prévia e importação (docs/inbox.md):
//! `inbox_id` (o item que chegou pela pasta de entrada) e `path` (um arquivo local, só dentro das
//! pastas permitidas: Downloads e a pasta de entrada) viram `files` antes da validação, com o
//! conteúdo lido aqui. Assim o agente externo não manda o arquivo em base64.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::tools;
use crate::api::Api;
use crate::i18n::{args, t, t_args};
use crate::import::inbox;

fn ids_of(v: Option<Value>) -> Vec<String> {
    match v {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect(),
        Some(Value::String(s)) => vec![s],
        _ => vec![],
    }
}

/// O caminho fica dentro de uma das pastas permitidas (resolvido, sem `..` nem atalhos)?
pub(crate) fn allowed(path: &Path, roots: &[PathBuf]) -> bool {
    let Ok(real) = path.canonicalize() else {
        return false;
    };
    roots
        .iter()
        .filter_map(|r| r.canonicalize().ok())
        .any(|r| real.starts_with(&r) && real != r)
}

fn read_path(api: &Api, raw: &str) -> Result<Value, String> {
    let inbox_dir = api
        .with_session(|s, _| inbox::folder(s.db.conn()))
        .ok()
        .flatten();
    let mut roots: Vec<PathBuf> = dirs::download_dir().into_iter().collect();
    roots.extend(inbox_dir.clone());
    let expanded = match raw.strip_prefix("~/") {
        Some(rest) => dirs::home_dir().map_or_else(|| PathBuf::from(raw), |h| h.join(rest)),
        None => PathBuf::from(raw),
    };
    if !allowed(&expanded, &roots) {
        return Err(t_args(
            "nucleo-caixa-erro-caminho",
            &args(&[
                ("caminho", raw.to_string().into()),
                (
                    "caixa",
                    inbox_dir
                        .map(|d| d.display().to_string())
                        .unwrap_or_default()
                        .into(),
                ),
            ]),
        ));
    }
    let meta = std::fs::metadata(&expanded).map_err(|e| {
        t_args(
            "nucleo-caixa-erro-caminho-arquivo",
            &args(&[
                ("caminho", raw.to_string().into()),
                ("erro", e.to_string().into()),
            ]),
        )
    })?;
    if meta.len() > inbox::MAX_BYTES {
        return Err(t("nucleo-caixa-erro-grande"));
    }
    let bytes = std::fs::read(&expanded).map_err(|e| {
        t_args(
            "nucleo-caixa-erro-caminho-arquivo",
            &args(&[
                ("caminho", raw.to_string().into()),
                ("erro", e.to_string().into()),
            ]),
        )
    })?;
    use base64::Engine;
    Ok(json!({
        "filename": expanded.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
        "content_base64": base64::engine::general_purpose::STANDARD.encode(bytes),
    }))
}

/// Troca `inbox_id` e `path` por `files`; devolve os ids da caixa usados (para marcar depois
/// de importar).
pub(crate) fn resolve(
    api: &Api,
    tool: &str,
    mut raw: Value,
) -> Result<(Value, Vec<String>), String> {
    let Some(obj) = raw.as_object_mut() else {
        return Ok((raw, vec![]));
    };
    let ids = ids_of(obj.remove("inbox_id"));
    let path = obj
        .remove("path")
        .and_then(|p| p.as_str().map(str::to_string));
    if ids.is_empty() && path.is_none() {
        return Ok((raw, vec![]));
    }
    if !tools::reads_turn_files(tool) || tools::reads_turn_texts(tool) {
        return Err(t("nucleo-caixa-erro-ferramenta"));
    }
    let mut files: Vec<Value> = match obj.remove("files") {
        Some(Value::Array(a)) => a,
        _ => vec![],
    };
    for id in &ids {
        let f = api
            .with_session(|s, _| inbox::file(s.db.conn(), id))
            .map_err(|e| e.message())?;
        files.push(json!({"filename": f.filename, "content_base64": f.content_base64}));
    }
    if let Some(p) = path {
        files.push(read_path(api, &p)?);
    }
    obj.insert("files".into(), Value::Array(files));
    Ok((raw, ids))
}

/// Depois de uma importação que deu certo, os itens da caixa usados saem da lista.
pub(crate) fn mark_imported(api: &Api, tool: &str, ids: &[String]) {
    if ids.is_empty() || !tool.starts_with("import_") {
        return;
    }
    let _ = api.with_session(|s, _| {
        for id in ids {
            inbox::set_status(s.db.conn(), id, inbox::InboxStatus::Imported)?;
        }
        Ok(())
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_paths_inside_the_allowed_folders() {
        let root = tempfile::tempdir().unwrap();
        let ok_dir = root.path().join("Inbox");
        std::fs::create_dir_all(&ok_dir).unwrap();
        std::fs::write(ok_dir.join("a.pdf"), b"%PDF").unwrap();
        std::fs::write(root.path().join("fora.pdf"), b"%PDF").unwrap();
        let roots = vec![ok_dir.clone()];
        assert!(allowed(&ok_dir.join("a.pdf"), &roots));
        assert!(!allowed(&root.path().join("fora.pdf"), &roots));
        // `..` não escapa da pasta.
        assert!(!allowed(&ok_dir.join("../fora.pdf"), &roots));
        // A própria pasta e o que não existe: não.
        assert!(!allowed(&ok_dir, &roots));
        assert!(!allowed(&ok_dir.join("nada.pdf"), &roots));
    }
}
