//! Mandar arquivos para a Lixeira do sistema (nunca apagar de vez): no macOS, `~/.Trash`; no
//! Linux, a lixeira do freedesktop (`~/.local/share/Trash`, com o `.trashinfo` para restaurar
//! pelo gerenciador de arquivos); nos demais, a pasta `removidos` dos dados do Treasury.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Pasta da lixeira (`fallback` quando o sistema não tem uma conhecida).
fn trash_dir(fallback: &Path) -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    if cfg!(target_os = "macos")
        && let Some(h) = &home
    {
        return h.join(".Trash");
    }
    if cfg!(all(unix, not(target_os = "macos"))) {
        let data = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| home.map(|h| h.join(".local").join("share")));
        if let Some(d) = data {
            return d.join("Trash");
        }
    }
    fallback.to_path_buf()
}

/// Nome livre na pasta (`cofre.db`, `cofre (2).db`…).
fn free_name(dir: &Path, name: &str) -> PathBuf {
    let first = dir.join(name);
    if !first.exists() {
        return first;
    }
    let (stem, ext) = match name.rsplit_once('.') {
        Some((s, e)) if !s.is_empty() => (s.to_string(), format!(".{e}")),
        _ => (name.to_string(), String::new()),
    };
    (2..)
        .map(|n| dir.join(format!("{stem} ({n}){ext}")))
        .find(|p| !p.exists())
        .unwrap_or(first)
}

/// `%` nos bytes fora do conjunto seguro de URL (para o `Path=` do `.trashinfo`).
fn url_path(p: &Path) -> String {
    p.to_string_lossy()
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"/-_.~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

/// Move o arquivo para a lixeira e devolve onde ficou.
pub fn move_to_trash(path: &Path, fallback: &Path) -> Result<PathBuf> {
    move_into(path, &trash_dir(fallback))
}

/// O mesmo, numa pasta de lixeira dada (testes).
pub fn move_into(path: &Path, trash: &Path) -> Result<PathBuf> {
    if !path.exists() {
        return Err(Error::not_found(crate::i18n::t_args(
            "nucleo-geral-arquivo",
            &crate::i18n::args(&[("caminho", path.display().to_string().into())]),
        )));
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| Error::invalid(crate::i18n::t("nucleo-geral-erro-caminho-sem-nome")))?;
    let freedesktop = trash.ends_with("Trash") && cfg!(all(unix, not(target_os = "macos")));
    let files = if freedesktop {
        trash.join("files")
    } else {
        trash.to_path_buf()
    };
    std::fs::create_dir_all(&files)?;
    let dest = free_name(&files, &name);
    if freedesktop {
        let info_dir = trash.join("info");
        std::fs::create_dir_all(&info_dir)?;
        let info_name = format!(
            "{}.trashinfo",
            dest.file_name().unwrap_or_default().to_string_lossy()
        );
        let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        std::fs::write(
            info_dir.join(info_name),
            format!(
                "[Trash Info]\nPath={}\nDeletionDate={}\n",
                url_path(&abs),
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S")
            ),
        )?;
    }
    if std::fs::rename(path, &dest).is_err() {
        // Outro volume: copia e só então remove o original.
        std::fs::copy(path, &dest)?;
        std::fs::remove_file(path)?;
    }
    Ok(dest)
}

#[cfg(test)]
mod tests {
    #[test]
    fn moves_and_never_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let trash = dir.path().join("lixeira");
        for _ in 0..2 {
            let f = dir.path().join("cofre.db");
            std::fs::write(&f, b"x").unwrap();
            super::move_into(&f, &trash).unwrap();
            assert!(!f.exists());
        }
        assert!(trash.join("cofre.db").exists());
        assert!(trash.join("cofre (2).db").exists());
        assert!(super::move_into(&dir.path().join("nada.db"), &trash).is_err());
    }
}
