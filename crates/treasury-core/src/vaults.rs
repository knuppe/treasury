//! Vários cofres (docs/vaults.md): cada cofre é um arquivo .db cifrado próprio, com senha,
//! desbloqueio rápido (keychain, PIN, chave física) e backups separados. A lista fica no
//! `config.toml`, fora dos cofres; o app abre um de cada vez.
//!
//! Sem lista no `config.toml` (quem já usava o app), existe só o **principal**, no caminho de
//! sempre (`db_path` do config ou `~/.treasury/data/treasury.db`), com os backups de sempre.
//! Os outros guardam os backups numa subpasta com o id deles.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::config::{Config, VaultEntry};
use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};

pub const PRINCIPAL: &str = "principal";

/// O cofre principal: o arquivo de sempre (com o nome no idioma da instalação até a lista ir
/// para o `config.toml`).
fn principal(cfg: &Config, root: &Path) -> VaultEntry {
    VaultEntry {
        id: PRINCIPAL.into(),
        name: t("nucleo-cofre-principal"),
        path: cfg
            .db_path
            .clone()
            .unwrap_or_else(|| root.join("data").join("treasury.db")),
        purpose: None,
        added_at: None,
        demo: false,
    }
}

/// Os cofres conhecidos (com o principal quando a lista está vazia).
pub fn entries(cfg: &Config, root: &Path) -> Vec<VaultEntry> {
    if cfg.vaults.is_empty() {
        vec![principal(cfg, root)]
    } else {
        cfg.vaults.clone()
    }
}

/// O cofre ativo (o escolhido, se ainda existir na lista; senão o primeiro).
pub fn active(cfg: &Config, root: &Path) -> VaultEntry {
    let all = entries(cfg, root);
    cfg.active_vault
        .as_deref()
        .and_then(|id| all.iter().find(|v| v.id == id).cloned())
        .unwrap_or_else(|| all[0].clone())
}

/// Pasta dos backups de um cofre: a de sempre para o principal, uma subpasta para os outros.
pub fn backups_dir(cfg: &Config, root: &Path, entry: &VaultEntry) -> PathBuf {
    let base = cfg
        .backups_dir
        .clone()
        .unwrap_or_else(|| root.join("backups"));
    if entry.id == PRINCIPAL {
        base
    } else {
        base.join(&entry.id)
    }
}

/// Um cofre na lista, como a tela mostra.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct VaultInfo {
    pub id: String,
    pub name: String,
    pub purpose: Option<String>,
    pub path: String,
    /// O arquivo existe (falso: cofre novo ainda sem criar, ou arquivo movido/apagado).
    pub exists: bool,
    pub active: bool,
    #[ts(type = "number | null")]
    pub bytes: Option<u64>,
    /// Demonstração sem senha (abre pela chave fixa do app).
    pub demo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct VaultList {
    pub active_id: String,
    pub vaults: Vec<VaultInfo>,
    /// Pasta padrão dos cofres novos.
    pub data_dir: String,
}

pub fn list(cfg: &Config, root: &Path) -> VaultList {
    let active = active(cfg, root);
    VaultList {
        active_id: active.id.clone(),
        vaults: entries(cfg, root)
            .into_iter()
            .map(|v| {
                let meta = std::fs::metadata(&v.path).ok();
                VaultInfo {
                    exists: meta.is_some(),
                    bytes: meta.map(|m| m.len()),
                    active: v.id == active.id,
                    path: v.path.to_string_lossy().into_owned(),
                    demo: v.demo,
                    id: v.id,
                    name: v.name,
                    purpose: v.purpose,
                }
            })
            .collect(),
        data_dir: root.join("data").to_string_lossy().into_owned(),
    }
}

/// Id a partir do nome: minúsculas sem acento, só letras, números e hífens; único na lista.
fn new_id(cfg: &Config, root: &Path, name: &str) -> String {
    let base: String = crate::import::text::fold(name)
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let base = if base.is_empty() {
        "cofre".to_string()
    } else {
        base
    };
    let taken: Vec<String> = entries(cfg, root).into_iter().map(|v| v.id).collect();
    let mut id = base.clone();
    let mut n = 2;
    while taken.contains(&id) {
        id = format!("{base}-{n}");
        n += 1;
    }
    id
}

fn not_found(id: &str) -> Error {
    Error::not_found(t_args(
        "nucleo-cofre-cofre",
        &args(&[("id", id.to_string().into())]),
    ))
}

fn with_path(key: &str, path: &Path) -> String {
    t_args(
        key,
        &args(&[("caminho", path.display().to_string().into())]),
    )
}

fn clean_name(name: &str) -> Result<String> {
    let n = name.trim();
    if n.is_empty() {
        return Err(Error::invalid(t("nucleo-cofre-erro-sem-nome")));
    }
    if n.chars().count() > 60 {
        return Err(Error::invalid(t_args(
            "nucleo-cofre-erro-nome-longo",
            &args(&[("maximo", 60.into())]),
        )));
    }
    Ok(n.to_string())
}

fn clean_purpose(p: Option<&str>) -> Option<String> {
    p.map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| p.chars().take(120).collect())
}

/// A lista passa a existir no config (com o principal) antes da primeira mudança.
fn materialize(cfg: &mut Config, root: &Path) {
    if cfg.vaults.is_empty() {
        cfg.vaults.push(principal(cfg, root));
    }
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(x), Ok(y)) => x == y,
        _ => a == b,
    }
}

/// Registra um arquivo .db que já existe (de outro computador, de um backup copiado…).
pub fn add_existing(
    cfg: &mut Config,
    root: &Path,
    path: &Path,
    name: Option<&str>,
    purpose: Option<&str>,
) -> Result<VaultEntry> {
    if !path.is_absolute() {
        return Err(Error::invalid(t("nucleo-cofre-erro-caminho-completo")));
    }
    let meta = std::fs::metadata(path)
        .map_err(|_| Error::invalid(with_path("nucleo-cofre-erro-arquivo-nao-encontrado", path)))?;
    if !meta.is_file() || meta.len() == 0 {
        return Err(Error::invalid(t("nucleo-cofre-erro-escolha-arquivo")));
    }
    // Um SQLite sem cifra começa com este cabeçalho; os cofres do Treasury nunca.
    let mut head = [0u8; 16];
    if let Ok(mut f) = std::fs::File::open(path) {
        use std::io::Read;
        let _ = f.read(&mut head);
    }
    if &head == b"SQLite format 3\0" {
        return Err(Error::invalid(t("nucleo-cofre-erro-sqlite-sem-cifra")));
    }
    if entries(cfg, root).iter().any(|v| same_file(&v.path, path)) {
        return Err(Error::conflict(t("nucleo-cofre-erro-arquivo-na-lista")));
    }
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| t("nucleo-cofre-nome-padrao"));
    let name = clean_name(name.unwrap_or(&stem))?;
    materialize(cfg, root);
    let entry = VaultEntry {
        id: new_id(cfg, root, &name),
        name,
        path: path.to_path_buf(),
        purpose: clean_purpose(purpose),
        added_at: Some(crate::time::now_utc()),
        demo: false,
    };
    cfg.vaults.push(entry.clone());
    Ok(entry)
}

/// Registra um cofre novo (o arquivo é criado depois, com a senha, em `vault.create`).
pub fn add_new(
    cfg: &mut Config,
    root: &Path,
    name: &str,
    purpose: Option<&str>,
    path: Option<&Path>,
) -> Result<VaultEntry> {
    let name = clean_name(name)?;
    materialize(cfg, root);
    let id = new_id(cfg, root, &name);
    let path = match path {
        Some(p) => {
            if !p.is_absolute() {
                return Err(Error::invalid(t("nucleo-cofre-erro-caminho-completo")));
            }
            p.to_path_buf()
        }
        None => root.join("data").join(format!("{id}.db")),
    };
    if path.exists() {
        return Err(Error::conflict(with_path(
            "nucleo-cofre-erro-arquivo-existe",
            &path,
        )));
    }
    if entries(cfg, root).iter().any(|v| same_file(&v.path, &path)) {
        return Err(Error::conflict(t("nucleo-cofre-erro-caminho-na-lista")));
    }
    let entry = VaultEntry {
        id,
        name,
        path,
        purpose: clean_purpose(purpose),
        added_at: Some(crate::time::now_utc()),
        demo: false,
    };
    cfg.vaults.push(entry.clone());
    Ok(entry)
}

pub fn update(
    cfg: &mut Config,
    root: &Path,
    id: &str,
    name: Option<&str>,
    purpose: Option<Option<&str>>,
) -> Result<()> {
    materialize(cfg, root);
    let v = cfg
        .vaults
        .iter_mut()
        .find(|v| v.id == id)
        .ok_or_else(|| not_found(id))?;
    if let Some(n) = name {
        v.name = clean_name(n)?;
    }
    if let Some(p) = purpose {
        v.purpose = clean_purpose(p);
    }
    Ok(())
}

/// Registra a cópia de um cofre: entrada nova, com arquivo próprio na pasta dos cofres (o
/// arquivo é copiado depois, por `vaults.copy`). Devolve a origem e a cópia.
pub fn add_copy(
    cfg: &mut Config,
    root: &Path,
    id: &str,
    name: Option<&str>,
) -> Result<(VaultEntry, VaultEntry)> {
    let src = entries(cfg, root)
        .into_iter()
        .find(|v| v.id == id)
        .ok_or_else(|| not_found(id))?;
    if !src.path.exists() {
        return Err(Error::invalid(t("nucleo-cofre-erro-nao-criado")));
    }
    let name = match name.map(str::trim).filter(|n| !n.is_empty()) {
        Some(n) => n.to_string(),
        None => {
            let base: String = src.name.chars().take(52).collect();
            t_args("nucleo-cofre-copia", &args(&[("nome", base.into())]))
        }
    };
    let mut copy = add_new(cfg, root, &name, src.purpose.as_deref(), None)?;
    // A cópia de um cofre de demonstração sem senha também abre pela chave fixa.
    if src.demo {
        mark_demo(cfg, &copy.id);
        copy.demo = true;
    }
    Ok((src, copy))
}

/// Marca o cofre como demonstração sem senha (abre pela chave fixa do app).
pub fn mark_demo(cfg: &mut Config, id: &str) {
    if let Some(v) = cfg.vaults.iter_mut().find(|v| v.id == id) {
        v.demo = true;
    }
}

/// Tira o cofre da lista (o arquivo fica onde está). O cofre aberto não sai.
pub fn remove(cfg: &mut Config, root: &Path, id: &str) -> Result<()> {
    if active(cfg, root).id == id {
        return Err(Error::invalid(t("nucleo-cofre-erro-em-uso")));
    }
    materialize(cfg, root);
    let before = cfg.vaults.len();
    cfg.vaults.retain(|v| v.id != id);
    if cfg.vaults.len() == before {
        return Err(not_found(id));
    }
    Ok(())
}

pub fn select(cfg: &mut Config, root: &Path, id: &str) -> Result<()> {
    if !entries(cfg, root).iter().any(|v| v.id == id) {
        return Err(not_found(id));
    }
    materialize(cfg, root);
    cfg.active_vault = Some(id.to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::api::{Actor, Api};
    use crate::paths::TreasuryPaths;

    const PASSWORD: &str = "senha-mestra-forte";

    fn create(api: &Api, family: &str) {
        api.call(
            "vault.create",
            json!({"password": PASSWORD, "household_name": family, "members": ["Ana"]}),
        )
        .unwrap();
    }

    fn accounts(api: &Api) -> Vec<String> {
        api.call("accounts.list", json!({}))
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["name"].as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn several_vaults_with_their_own_data_and_backups() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".t");
        let api = Api::new_unlocked_process(TreasuryPaths::with_root(&root).unwrap()).unwrap();
        create(&api, "Família");
        api.call(
            "accounts.create",
            json!({"name": "Conta da família", "kind": "checking"}),
        )
        .unwrap();
        let st = api.call("vault.status", json!({})).unwrap();
        assert_eq!(
            (st["vault_id"].clone(), st["vault_count"].clone()),
            (json!("principal"), json!(1))
        );

        // Cofre novo: o atual fecha e o novo fica pronto para criar.
        let st = api
            .call(
                "vaults.new",
                json!({"name": "Empresa Ácme", "purpose": "PJ"}),
            )
            .unwrap();
        assert_eq!(st["state"], "no_vault");
        assert_eq!(
            (st["vault_id"].clone(), st["vault_name"].clone()),
            (json!("empresa-acme"), json!("Empresa Ácme"))
        );
        assert!(
            std::path::Path::new(st["db_path"].as_str().unwrap()).ends_with("data/empresa-acme.db")
        );
        create(&api, "Empresa");
        api.call(
            "accounts.create",
            json!({"name": "Conta PJ", "kind": "checking"}),
        )
        .unwrap();
        assert_eq!(accounts(&api), ["Conta PJ"]);
        api.call("backup.create", json!({})).unwrap();
        assert!(root.join("backups/empresa-acme").is_dir());

        // Volta ao principal: bloqueia, pede a senha dele e mostra só os dados dele.
        let st = api
            .call("vaults.select", json!({"id": "principal"}))
            .unwrap();
        assert_eq!(st["state"], "locked");
        api.call("vault.unlock", json!({"password": PASSWORD}))
            .unwrap();
        assert_eq!(accounts(&api), ["Conta da família"]);
        let list = api.call("vaults.list", json!({})).unwrap();
        assert_eq!(list["active_id"], "principal");
        assert_eq!(list["vaults"].as_array().unwrap().len(), 2);

        // Arquivo que já existe: entra na lista; o mesmo arquivo duas vezes, não.
        let copy = dir.path().join("copia.db");
        std::fs::copy(root.join("data/empresa-acme.db"), &copy).unwrap();
        let st = api
            .call(
                "vaults.add",
                json!({"path": copy.to_string_lossy(), "name": "Cópia", "select": false}),
            )
            .unwrap();
        assert_eq!(st["vault_count"], 3);
        assert!(
            api.call("vaults.add", json!({"path": copy.to_string_lossy()}))
                .is_err()
        );
        assert!(
            api.call("vaults.add", json!({"path": "relativo.db"}))
                .is_err()
        );
        let plain = dir.path().join("plain.db");
        std::fs::write(&plain, b"SQLite format 3\0resto").unwrap();
        assert!(
            api.call("vaults.add", json!({"path": plain.to_string_lossy()}))
                .is_err()
        );

        // Renomear, tirar da lista (o arquivo fica) e não tirar o que está em uso.
        api.call(
            "vaults.update",
            json!({"id": "copia", "name": "Cópia antiga", "purpose": null}),
        )
        .unwrap();
        assert!(
            api.call("vaults.remove", json!({"id": "principal"}))
                .is_err()
        );
        api.call("vaults.remove", json!({"id": "copia"})).unwrap();
        assert!(copy.exists());
        assert_eq!(
            api.call("vaults.list", json!({})).unwrap()["vaults"]
                .as_array()
                .unwrap()
                .len(),
            2
        );

        // Abrir o app de novo começa no último cofre escolhido.
        api.call("vaults.select", json!({"id": "empresa-acme"}))
            .unwrap();
        let again = TreasuryPaths::with_root(&root).unwrap();
        assert_eq!(again.vault_id, "empresa-acme");
        // Só a interface gerencia cofres.
        let mcp = Actor::Mcp {
            client: "x".into(),
            read_only: false,
        };
        assert!(api.call_as(&mcp, "vaults.list", json!({})).is_err());
    }

    #[test]
    fn copy_and_remove_to_trash() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".t");
        let api = Api::new_unlocked_process(TreasuryPaths::with_root(&root).unwrap()).unwrap();
        create(&api, "Família");
        api.call(
            "accounts.create",
            json!({"name": "Conta da família", "kind": "checking"}),
        )
        .unwrap();
        let names = |l: &serde_json::Value| -> Vec<String> {
            l["vaults"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v["name"].as_str().unwrap().to_string())
                .collect()
        };
        // Cópia do cofre aberto: mesma senha, mesmos dados; continua no principal.
        let l = api.call("vaults.copy", json!({"id": "principal"})).unwrap();
        assert_eq!(names(&l), ["Principal", "Principal (cópia)"]);
        assert_eq!(l["active_id"], "principal");
        let copy = l["vaults"][1].clone();
        assert_eq!(copy["exists"], true);
        // Cópia de um cofre fechado (a cópia), com outro nome.
        let l = api
            .call("vaults.copy", json!({"id": copy["id"], "name": "Teste"}))
            .unwrap();
        assert_eq!(names(&l), ["Principal", "Principal (cópia)", "Teste"]);
        api.call("vaults.select", json!({"id": copy["id"]}))
            .unwrap();
        api.call("vault.unlock", json!({"password": PASSWORD}))
            .unwrap();
        assert_eq!(accounts(&api), ["Conta da família"]);

        // O cofre em uso não sai; os outros saem, com o arquivo na Lixeira.
        assert!(
            api.call(
                "vaults.remove",
                json!({"id": copy["id"], "delete_file": true})
            )
            .is_err()
        );
        let teste = l["vaults"][2].clone();
        let path = std::path::PathBuf::from(teste["path"].as_str().unwrap());
        assert!(path.exists());
        let l = api
            .call(
                "vaults.remove",
                json!({"id": teste["id"], "delete_file": true}),
            )
            .unwrap();
        assert_eq!(names(&l), ["Principal", "Principal (cópia)"]);
        assert!(!path.exists());
        assert!(root.join("lixeira-testes").join("teste.db").exists());
        // Tirar só da lista deixa o arquivo.
        let principal = std::path::PathBuf::from(l["vaults"][0]["path"].as_str().unwrap());
        api.call("vaults.remove", json!({"id": "principal"}))
            .unwrap();
        assert!(principal.exists());
        assert!(
            api.call("vaults.copy", json!({"id": "nao-existe"}))
                .is_err()
        );
    }

    /// Nomes que o app dá aos cofres e os erros no idioma da instalação.
    #[test]
    fn names_and_errors_follow_the_installation_locale() {
        use crate::i18n::{Locale, set_current};
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".t");
        let api = Api::new_unlocked_process(TreasuryPaths::with_root(&root).unwrap()).unwrap();
        create(&api, "Family");
        set_current(Locale::En);
        let l = api.call("vaults.copy", json!({"id": "principal"}));
        let e = api
            .call("vaults.copy", json!({"id": "nao-existe"}))
            .unwrap_err()
            .message();
        let long = api
            .call("vaults.new", json!({"name": "x".repeat(61)}))
            .unwrap_err()
            .message();
        set_current(Locale::PtBr);
        let l = l.unwrap();
        let names: Vec<&str> = l["vaults"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["Main", "Main (copy)"]);
        assert_eq!(e, "not found: vault nao-existe");
        assert_eq!(long, "vault name too long (maximum 60 characters)");
    }
}
