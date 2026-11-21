//! CLIs locais como motor (Claude Code, Codex, Gemini CLI, Grok, OpenCode, personalizada): o loop do
//! agente é o da própria CLI. O app:
//!
//! 1. cria um diretório de trabalho temporário e vazio (`0700`) e roda a CLI nele;
//! 2. registra um token de sessão só deste job e grava, fora do diretório de trabalho, um
//!    arquivo `0600` com ele e a configuração MCP no formato da CLI, apontando para o
//!    executável atual com o subcomando `mcp` (o socket aceita o token como "agente do
//!    app": escritas viram propostas);
//! 3. expande o template (`{prompt}`, `{mcp_config}`, `{model}`, `{workdir}`; trechos entre
//!    `[` e `]` somem quando um marcador dentro deles está vazio) sem shell;
//! 4. lê a saída em streaming (`stream-json` do Claude Code, eventos JSON do OpenCode; texto
//!    nas outras) e apaga tudo
//!    no fim. Cancelar mata o processo.
//!
//! As ferramentas nativas da CLI (arquivos, shell, web) ficam desligadas por padrão e só o
//! MCP do Treasury fica liberado. Isso vale mesmo com o template editado: depois da
//! expansão, [`restrict_argv`] tira as flags que liberariam o computador e acrescenta as
//! restrições de cada CLI conhecida. Com o acesso ao computador liberado pelo usuário na
//! conversa ([`CliPolicy::computer_access`]), só a leitura de arquivos volta (Claude Code:
//! `Read`, `Glob`, `Grep` na pasta pessoal; Gemini CLI: as ferramentas de leitura); shell,
//! escrita e web continuam bloqueados.

use std::collections::HashMap;
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use ts_rs::TS;

use super::engine::{ALL_PROVIDERS, Engine, Protocol, Provider};
use super::usage::Tokens;
use crate::error::{Error, Result};
use crate::i18n;

/// Tempo máximo padrão de uma execução de CLI (ajustável em Ajustes › IA:
/// `ai::settings::CLI_TIMEOUT_CHOICES`).
pub const CLI_TIMEOUT: Duration = Duration::from_secs(15 * 60);

// ---------------------------------------------------------------------------
// Template

/// Divide o template em argumentos (aspas simples e duplas, sem shell). Os grupos
/// opcionais `[ ... ]` ficam marcados com `\u{1}`/`\u{2}` para a expansão.
pub fn split_template(t: &str) -> Result<Vec<String>> {
    let mut out = vec![];
    let mut cur = String::new();
    let mut has = false;
    let mut quote: Option<char> = None;
    let mut depth = 0;
    for c in t.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), c) => cur.push(c),
            (None, '"' | '\'') => {
                quote = Some(c);
                has = true;
            }
            (None, '[') => {
                if has {
                    out.push(std::mem::take(&mut cur));
                    has = false;
                }
                depth += 1;
                out.push("\u{1}".into());
            }
            (None, ']') => {
                if depth == 0 {
                    return Err(Error::invalid(i18n::t(
                        "nucleo-ia-template-colchete-sem-abrir",
                    )));
                }
                if has {
                    out.push(std::mem::take(&mut cur));
                    has = false;
                }
                depth -= 1;
                out.push("\u{2}".into());
            }
            (None, c) if c.is_whitespace() => {
                if has {
                    out.push(std::mem::take(&mut cur));
                    has = false;
                }
            }
            (None, c) => {
                cur.push(c);
                has = true;
            }
        }
    }
    if quote.is_some() {
        return Err(Error::invalid(i18n::t("nucleo-ia-template-aspas")));
    }
    if depth != 0 {
        return Err(Error::invalid(i18n::t(
            "nucleo-ia-template-colchete-sem-fechar",
        )));
    }
    if has {
        out.push(cur);
    }
    Ok(out)
}

/// Expande os marcadores. Um grupo `[ ... ]` some inteiro se algum marcador dentro dele
/// estiver vazio. Marcadores desconhecidos ficam como estão.
pub fn expand(template: &str, vars: &HashMap<&str, String>) -> Result<Vec<String>> {
    let tokens = split_template(template)?;
    let mut stack: Vec<(Vec<String>, bool)> = vec![(vec![], true)];
    for tok in tokens {
        match tok.as_str() {
            "\u{1}" => stack.push((vec![], true)),
            "\u{2}" => {
                let (items, ok) = stack.pop().expect("grupo aberto");
                let top = stack.last_mut().expect("raiz");
                if ok {
                    top.0.extend(items);
                }
            }
            _ => {
                let (s, empty) = substitute(&tok, vars);
                let top = stack.last_mut().expect("raiz");
                if empty {
                    top.1 = false;
                }
                top.0.push(s);
            }
        }
    }
    let (args, _) = stack.pop().expect("raiz");
    if args.is_empty() {
        return Err(Error::invalid(i18n::t("nucleo-ia-template-vazio")));
    }
    Ok(args)
}

/// Troca os marcadores numa passada só: um valor (o prompt, por exemplo) que contenha
/// `{model}` não é expandido de novo.
fn substitute(tok: &str, vars: &HashMap<&str, String>) -> (String, bool) {
    let mut out = String::new();
    let mut empty = false;
    let mut rest = tok;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('}').map(|end| (&after[..end], end)) {
            Some((name, end)) if vars.contains_key(name) => {
                let v = &vars[name];
                empty |= v.is_empty();
                out.push_str(v);
                rest = &after[end + 1..];
            }
            _ => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    (out, empty)
}

// ---------------------------------------------------------------------------
// PATH e detecção

/// PATH do processo mais as pastas comuns de instalação: apps de interface no macOS
/// recebem um PATH mínimo e não achariam CLIs instaladas por npm, Homebrew ou instaladores.
pub fn search_path() -> OsString {
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    if let Some(h) = dirs::home_dir() {
        for d in [
            ".local/bin",
            ".claude/local",
            ".npm-global/bin",
            ".bun/bin",
            ".volta/bin",
            ".grok/bin",
            ".cargo/bin",
        ] {
            dirs.push(h.join(d));
        }
    }
    for d in ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin", "/bin"] {
        dirs.push(PathBuf::from(d));
    }
    let mut seen = std::collections::HashSet::new();
    dirs.retain(|d| seen.insert(d.clone()));
    std::env::join_paths(dirs).unwrap_or_default()
}

pub fn find_executable(name: &str) -> Option<PathBuf> {
    let p = Path::new(name);
    if p.components().count() > 1 {
        return is_executable(p).then(|| p.to_path_buf());
    }
    std::env::split_paths(&search_path())
        .map(|d| d.join(name))
        .find(|c| is_executable(c))
}

fn is_executable(p: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(p).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        p.is_file()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CliDetection {
    pub provider: Provider,
    pub label: String,
    pub binary: String,
    /// Caminho encontrado (ausente = não instalada).
    pub path: Option<String>,
    pub version: Option<String>,
    /// Indício de login (arquivo de credenciais da CLI); `null` = não dá para saber sem
    /// executar a CLI.
    pub logged_in: Option<bool>,
    pub error: Option<String>,
}

/// Roda um comando curto com tempo-limite e devolve o stdout.
fn run_short(path: &Path, args: &[&str], timeout: Duration) -> std::result::Result<String, String> {
    let mut child = Command::new(path)
        .args(args)
        .env("PATH", search_path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("tempo esgotado".into());
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => return Err(e.to_string()),
        }
    }
    let mut out = String::new();
    if let Some(mut so) = child.stdout.take() {
        let _ = so.by_ref().take(64 * 1024).read_to_string(&mut out);
    }
    Ok(out)
}

fn login_hint(p: Provider) -> Option<bool> {
    let home = dirs::home_dir()?;
    let files: &[&str] = match p {
        Provider::Codex => &[".codex/auth.json"],
        Provider::GeminiCli => &[".gemini/oauth_creds.json", ".gemini/google_accounts.json"],
        Provider::ClaudeCode if !cfg!(target_os = "macos") => &[".claude/.credentials.json"],
        // OpenCode: chaves em `auth.json`; sem ele, o provedor pode vir da configuração ou de
        // variável de ambiente, então a falta do arquivo não diz nada.
        Provider::OpencodeCli => {
            let data = std::env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".local/share"));
            return data.join("opencode/auth.json").exists().then_some(true);
        }
        // No macOS o Claude Code guarda a credencial no keychain.
        _ => return None,
    };
    Some(files.iter().any(|f| home.join(f).exists()))
}

/// Detecta as CLIs conhecidas no PATH, com versão (`--version`).
pub fn detect() -> Vec<CliDetection> {
    ALL_PROVIDERS
        .iter()
        .filter_map(|&p| p.binary().map(|b| (p, b)))
        .map(|(p, bin)| {
            let path = find_executable(bin);
            let (version, error) = match &path {
                Some(path) => match run_short(path, &["--version"], Duration::from_secs(8)) {
                    Ok(v) => (
                        v.lines()
                            .find(|l| !l.trim().is_empty())
                            .map(|l| l.trim().chars().take(80).collect()),
                        None,
                    ),
                    Err(e) => (None, Some(e)),
                },
                None => (None, None),
            };
            CliDetection {
                provider: p,
                label: p.label().into(),
                binary: bin.into(),
                path: path.map(|p| p.to_string_lossy().into_owned()),
                version,
                logged_in: login_hint(p),
                error,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Configuração MCP por CLI

/// Servidor MCP do Treasury para a CLI: o executável atual com `mcp`, a raiz de dados
/// (fora do padrão) e o arquivo com o token de sessão do job.
pub struct McpTarget<'a> {
    pub command: &'a Path,
    pub home: Option<&'a Path>,
    pub token_file: &'a Path,
}

fn server_json(t: &McpTarget<'_>) -> Value {
    let mut env = serde_json::Map::new();
    env.insert(
        crate::mcp::TOKEN_FILE_ENV.into(),
        json!(t.token_file.to_string_lossy()),
    );
    if let Some(h) = t.home {
        env.insert("TREASURY_HOME".into(), json!(h.to_string_lossy()));
    }
    json!({
        "command": t.command.to_string_lossy(),
        "args": ["mcp"],
        "env": env,
    })
}

fn toml_str(s: &str) -> String {
    let escaped: String = s
        .chars()
        .flat_map(|c| match c {
            '"' => vec!['\\', '"'],
            '\\' => vec!['\\', '\\'],
            c => vec![c],
        })
        .collect();
    format!("\"{escaped}\"")
}

/// Ferramentas nativas da Gemini CLI que nunca são liberadas (shell, escrita, web, memória).
const GEMINI_ALWAYS_EXCLUDED: &[&str] = &[
    "run_shell_command",
    "write_file",
    "replace",
    "web_fetch",
    "google_web_search",
    "save_memory",
];
/// Ferramentas de leitura da Gemini CLI: liberadas só com o acesso ao computador.
const GEMINI_READ_TOOLS: &[&str] = &[
    "read_file",
    "read_many_files",
    "list_directory",
    "glob",
    "search_file_content",
];

/// O que a CLI pode fazer além do MCP do Treasury nesta conversa.
#[derive(Debug, Clone, Default)]
pub struct CliPolicy {
    /// O usuário liberou nesta conversa a leitura de arquivos do computador.
    pub computer_access: bool,
    /// Pasta pessoal (liberada para leitura quando `computer_access`).
    pub home: Option<PathBuf>,
    /// MCPs que o usuário já tem configurados na CLI (Grok, Codex): ficam de fora desta
    /// execução (o Claude Code usa `--strict-mcp-config`; o Gemini, a lista de permitidos).
    pub user_mcps: Vec<String>,
    /// OpenCode 1.x: sem `--standalone` e com o formato antigo de MCP (ver [`opencode_v1`]).
    pub opencode_v1: bool,
    /// MCPs que o usuário já tem nas CLIs e liberou para o agente deste cofre (Ajustes › IA ›
    /// MCPs das suas CLIs; padrão: nenhum). Os outros ficam desligados na conversa.
    pub allowed_mcps: Vec<String>,
    /// Claude Code (`--strict-mcp-config`): as definições dos liberados, copiadas do
    /// `~/.claude.json`, para entrarem no arquivo de MCP da conversa.
    pub claude_user_mcps: serde_json::Map<String, Value>,
}

impl CliPolicy {
    fn allowed(&self, name: &str) -> bool {
        self.allowed_mcps.iter().any(|n| n == name)
    }
}

impl CliPolicy {
    fn gemini_excluded(&self) -> Vec<&'static str> {
        let mut v = GEMINI_ALWAYS_EXCLUDED.to_vec();
        if !self.computer_access {
            v.extend_from_slice(GEMINI_READ_TOOLS);
        }
        v
    }
}

/// Ferramentas nativas do Claude Code negadas por nome, além de `--tools ""` (defesa em
/// profundidade caso uma versão trate `--tools` de outro jeito).
const CLAUDE_DENIED: &[&str] = &[
    "Bash",
    "Edit",
    "Write",
    "NotebookEdit",
    "WebFetch",
    "WebSearch",
];
/// Ferramentas de leitura do Claude Code liberadas com o acesso ao computador.
const CLAUDE_READ_TOOLS: &str = "Read,Glob,Grep";

/// Quantos valores uma flag removida leva consigo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Removed {
    /// Flag sem valor.
    Switch,
    /// Um valor (`--sandbox read-only`).
    One,
    /// Vários, até a próxima flag (`--allowedTools a b c`).
    Many,
}

/// Flags que liberariam o computador e por isso saem do comando de cada CLI conhecida.
fn removed_flag(provider: Provider, flag: &str) -> Option<Removed> {
    use Removed::*;
    Some(match (provider, flag) {
        (
            Provider::ClaudeCode,
            "--tools" | "--allowedTools" | "--allowed-tools" | "--disallowedTools"
            | "--disallowed-tools" | "--add-dir",
        ) => Many,
        (Provider::ClaudeCode, "--permission-mode" | "--setting-sources") => One,
        (
            Provider::ClaudeCode,
            "--dangerously-skip-permissions" | "--allow-dangerously-skip-permissions",
        ) => Switch,
        (Provider::Codex, "--sandbox" | "-s" | "--add-dir") => One,
        (
            Provider::Codex,
            "--dangerously-bypass-approvals-and-sandbox" | "--full-auto" | "--yolo",
        ) => Switch,
        (Provider::GeminiCli, "--approval-mode" | "--include-directories") => One,
        (Provider::GeminiCli, "--allowed-mcp-server-names") => Many,
        (Provider::GeminiCli, "--yolo" | "-y") => Switch,
        (
            Provider::GrokCli,
            "--allow" | "--allowedTools" | "--allowed-tools" | "--permission-mode" | "--sandbox"
            | "--cwd",
        ) => One,
        (Provider::GrokCli, "--always-approve" | "--dangerously-skip-permissions") => Switch,
        // OpenCode: outro servidor, outra sessão, outro agente, arquivo do disco, outro formato.
        (
            Provider::OpencodeCli,
            "--server" | "--attach" | "--port" | "--session" | "-s" | "--agent" | "--file" | "-f"
            | "--format",
        ) => One,
        (
            Provider::OpencodeCli,
            "--auto"
            | "--continue"
            | "-c"
            | "--fork"
            | "--share"
            | "--standalone"
            | "--dangerously-skip-permissions",
        ) => Switch,
        _ => return None,
    })
}

/// Aplica a política de ferramentas ao comando já expandido da CLI: tira as flags que
/// liberariam o computador (inclusive as que o usuário pôs no template) e acrescenta as
/// restrições. A CLI personalizada fica como está: o template é do usuário, e só as
/// instruções pedem para não olhar o computador.
pub fn restrict_argv(provider: Provider, argv: Vec<String>, policy: &CliPolicy) -> Vec<String> {
    if !matches!(
        provider,
        Provider::ClaudeCode
            | Provider::Codex
            | Provider::GeminiCli
            | Provider::GrokCli
            | Provider::OpencodeCli
    ) {
        return argv;
    }
    let mut out: Vec<String> = Vec::with_capacity(argv.len() + 16);
    let mut it = argv.into_iter();
    if let Some(program) = it.next() {
        out.push(program);
    }
    let mut skip: Option<Removed> = None;
    for a in it {
        match skip {
            Some(Removed::One) => {
                skip = None;
                continue;
            }
            Some(Removed::Many) if !a.starts_with('-') => continue,
            _ => skip = None,
        }
        // `--flag=valor` some inteiro.
        let (flag, inline) = match a.split_once('=') {
            Some((f, _)) if f.starts_with("--") => (f, true),
            _ => (a.as_str(), false),
        };
        match removed_flag(provider, flag) {
            Some(Removed::Switch) => continue,
            Some(r) => {
                if !inline {
                    skip = Some(r);
                }
                continue;
            }
            None => out.push(a),
        }
    }
    let home = policy
        .home
        .as_ref()
        .map(|h| h.to_string_lossy().into_owned());
    match provider {
        Provider::ClaudeCode => {
            if !out.iter().any(|a| a == "--strict-mcp-config") {
                out.push("--strict-mcp-config".into());
            }
            // Só as configurações do projeto (a pasta de trabalho vazia do Treasury): ficam de
            // fora hooks, plugins e permissões do usuário, que poderiam rodar comandos.
            out.push("--setting-sources".into());
            out.push("project".into());
            // Sem histórico da sessão em ~/.claude: o que as ferramentas devolveram (valores,
            // lançamentos) ficaria lá em texto puro, fora do cofre (docs/local-security.md,
            // item 10). A conversa continua no cofre; o Treasury não retoma sessões da CLI.
            if out.iter().any(|a| a == "-p" || a == "--print")
                && !out.iter().any(|a| a == "--no-session-persistence")
            {
                out.push("--no-session-persistence".into());
            }
            // `--tools ""` tira todas as ferramentas nativas; as do MCP continuam.
            out.push("--tools".into());
            out.push(if policy.computer_access {
                CLAUDE_READ_TOOLS.into()
            } else {
                String::new()
            });
            out.push("--disallowedTools".into());
            out.extend(CLAUDE_DENIED.iter().map(|t| t.to_string()));
            out.push("--allowedTools".into());
            out.push("mcp__treasury__*".into());
            for name in policy.claude_user_mcps.keys() {
                out.push(format!("mcp__{name}__*"));
            }
            if policy.computer_access {
                out.extend(CLAUDE_READ_TOOLS.split(',').map(str::to_string));
                if let Some(h) = home {
                    out.push("--add-dir".into());
                    out.push(h);
                }
            }
        }
        Provider::Codex => {
            // MCPs do usuário no ~/.codex/config.toml ficam desligados nesta execução.
            for name in &policy.user_mcps {
                if name != crate::mcp::SERVER_NAME && is_plain_key(name) && !policy.allowed(name) {
                    out.insert(
                        1.min(out.len()),
                        format!("mcp_servers.{name}.enabled=false"),
                    );
                    out.insert(1.min(out.len()), "-c".into());
                }
            }
            // Somente leitura e sem rede, logo depois do subcomando `exec`.
            let at = out.iter().position(|a| a == "exec").map_or(1, |i| i + 1);
            let at = at.min(out.len());
            out.insert(at, "read-only".into());
            out.insert(at, "--sandbox".into());
            // O sandbox somente leitura ainda lê qualquer arquivo pelo shell (assessment S2):
            // sem o acesso ao computador liberado, a Codex fica sem o shell e sem abrir
            // imagens do disco; sobram as ferramentas do Treasury.
            if !policy.computer_access {
                let at = out
                    .iter()
                    .position(|a| a == "exec")
                    .unwrap_or(1)
                    .min(out.len());
                for kv in CODEX_NO_DISK.iter().rev() {
                    out.insert(at, (*kv).into());
                    out.insert(at, "-c".into());
                }
            }
        }
        Provider::GeminiCli => {
            // Só o Treasury e os MCPs que o usuário liberou (a lista do template sai acima).
            let at = 1.min(out.len());
            for name in policy
                .allowed_mcps
                .iter()
                .rev()
                .chain(std::iter::once(&crate::mcp::SERVER_NAME.to_string()))
            {
                out.insert(at, name.clone());
                out.insert(at, "--allowed-mcp-server-names".into());
            }
            if policy.computer_access
                && let Some(h) = home
            {
                let at = 1.min(out.len());
                out.insert(at, h);
                out.insert(at, "--include-directories".into());
            }
        }
        Provider::GrokCli => {
            // O MCP do Treasury vem do `.grok/config.toml` da pasta de trabalho (fixa e só do
            // Treasury), que a CLI só lê em pasta confiável: `--trust` registra essa pasta, uma
            // vez, em ~/.grok/trusted_folders.toml.
            let at = 1.min(out.len());
            let mut extra: Vec<String> = vec!["--trust".into(), "--disable-web-search".into()];
            extra.push("--allow".into());
            extra.push(format!("MCPTool({}__*)", crate::mcp::SERVER_NAME));
            for t in GROK_DENIED {
                if policy.computer_access && GROK_READ_TOOLS.contains(t) {
                    continue;
                }
                extra.push("--deny".into());
                extra.push((*t).into());
            }
            // MCPs que o usuário configurou na CLI (ou que ela importa do Claude e do Cursor)
            // ficam de fora desta conversa, a não ser os que ele liberou para o agente.
            for name in &policy.user_mcps {
                if name == crate::mcp::SERVER_NAME {
                    continue;
                }
                extra.push(
                    if policy.allowed(name) {
                        "--allow"
                    } else {
                        "--deny"
                    }
                    .into(),
                );
                extra.push(format!("MCPTool({name}__*)"));
            }
            for (i, a) in extra.into_iter().enumerate() {
                out.insert(at + i, a);
            }
        }
        Provider::OpencodeCli => {
            // Servidor próprio desta execução (2.x; o serviço em segundo plano não leria a
            // configuração do Treasury) e eventos JSON, logo depois do subcomando `run`. A
            // 1.x já roda um servidor por execução e não conhece `--standalone`.
            let at = out.iter().position(|a| a == "run").map_or(1, |i| i + 1);
            let at = at.min(out.len());
            out.insert(at, "json".into());
            out.insert(at, "--format".into());
            if !policy.opencode_v1 {
                out.insert(at, "--standalone".into());
            }
        }
        _ => {}
    }
    out
}

/// Ferramentas nativas do OpenCode liberadas com o acesso ao computador (só leitura).
const OPENCODE_READ_TOOLS: &[&str] = &["read", "glob", "grep", "list", "external_directory"];

/// Configuração do OpenCode desta execução, em `OPENCODE_CONFIG_CONTENT` (vale mais que a
/// global e a do projeto). Tudo negado, menos as ferramentas do Treasury: na 1.x elas se
/// chamam `treasury_<nome>`; na 2.x o modelo as chama de dentro do `execute` (Code Mode,
/// `tools.treasury.<nome>()`), que roda sem disco, shell nem rede, e cada chamada passa pela
/// mesma permissão (`treasury.<nome>`). O agente padrão (`build`) recebe as mesmas regras,
/// porque as do agente valem mais que as gerais. MCPs do usuário ficam desligados.
fn opencode_config(server: &Value, policy: &CliPolicy) -> Value {
    let name = crate::mcp::SERVER_NAME;
    let mut permission = serde_json::Map::new();
    permission.insert("*".into(), json!("deny"));
    permission.insert("execute".into(), json!("allow"));
    permission.insert(format!("{name}*"), json!("allow"));
    for n in &policy.allowed_mcps {
        permission.insert(format!("{n}*"), json!("allow"));
    }
    if policy.computer_access {
        for t in OPENCODE_READ_TOOLS {
            permission.insert((*t).into(), json!("allow"));
        }
    }
    let command = json!([server["command"], "mcp"]);
    let mut servers = serde_json::Map::new();
    let mcp = if policy.opencode_v1 {
        servers.insert(
            name.into(),
            json!({"type": "local", "command": command, "environment": server["env"], "enabled": true}),
        );
        for n in policy
            .user_mcps
            .iter()
            .filter(|n| *n != name && !policy.allowed(n))
        {
            servers.insert(n.clone(), json!({"enabled": false}));
        }
        Value::Object(servers)
    } else {
        servers.insert(
            name.into(),
            json!({"type": "local", "command": command, "environment": server["env"]}),
        );
        for n in policy
            .user_mcps
            .iter()
            .filter(|n| *n != name && !policy.allowed(n))
        {
            servers.insert(
                n.clone(),
                json!({"type": "local", "command": ["true"], "disabled": true}),
            );
        }
        json!({"servers": servers})
    };
    json!({
        "share": "disabled",
        "autoupdate": false,
        "default_agent": "build",
        "permission": permission,
        "agent": {"build": {"permission": permission}},
        "mcp": mcp,
    })
}

/// Lembrete no começo do prompt do OpenCode 2.x: as ferramentas de MCP ficam dentro do
/// `execute` e às vezes ainda não aparecem no catálogo do primeiro passo (o servidor termina
/// de conectar logo depois); sem isso, cerca de 1 em 4 respostas dizia não ter ferramentas
/// (5 de 5 certas com ele, 2.0.32).
pub const OPENCODE_TOOLS_HINT: &str = "The Treasury tools come from the MCP server \"treasury\". In OpenCode, call them from inside the `execute` tool, as `return await tools.treasury.<tool_name>({...})`; the tool list may load a moment after you start, so call `execute` even if the catalog looks empty.";

/// OpenCode 1.x instalado? (`opencode --version`: "1.2.3" ou "opencode v2.0.32"; até 8 s).
pub fn opencode_v1() -> bool {
    let Some(exe) = find_executable("opencode") else {
        return false;
    };
    run_short(&exe, &["--version"], Duration::from_secs(8))
        .map(|v| opencode_major(&v).is_some_and(|m| m < 2))
        .unwrap_or(false)
}

fn opencode_major(version: &str) -> Option<u32> {
    let word = version.split_whitespace().last()?;
    word.trim_start_matches('v').split('.').next()?.parse().ok()
}

/// Nomes dos MCPs da configuração global do OpenCode (`~/.config/opencode` ou
/// `$XDG_CONFIG_HOME/opencode`: `opencode.json`, `opencode.jsonc`, `config.json`; e o
/// arquivo de `$OPENCODE_CONFIG`), só para desligá-los nesta execução. Nada é alterado.
pub fn opencode_config_mcps(home: Option<&Path>) -> Vec<String> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| home.map(|h| h.join(".config")))
        .map(|b| b.join("opencode"));
    let mut files: Vec<PathBuf> = base
        .map(|b| {
            ["opencode.json", "opencode.jsonc", "config.json"]
                .iter()
                .map(|f| b.join(f))
                .collect()
        })
        .unwrap_or_default();
    if let Some(p) = std::env::var_os("OPENCODE_CONFIG") {
        files.push(PathBuf::from(p));
    }
    let mut names = vec![];
    for f in files {
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        for n in opencode_mcp_names(&text) {
            if !names.contains(&n) {
                names.push(n);
            }
        }
    }
    names
}

/// Servidores em `mcp` (formato da 1.x) ou `mcp.servers` (2.x), num JSON com comentários.
fn opencode_mcp_names(text: &str) -> Vec<String> {
    let Ok(v) = serde_json::from_str::<Value>(&strip_json_comments(text)) else {
        return vec![];
    };
    let Some(mcp) = v["mcp"].as_object() else {
        return vec![];
    };
    let mut names = vec![];
    for (k, val) in mcp {
        if k == "servers" && val.get("type").is_none() {
            names.extend(val.as_object().into_iter().flat_map(|m| m.keys().cloned()));
        } else {
            names.push(k.clone());
        }
    }
    names
}

/// Tira comentários `//` e `/* */` (fora de strings) e vírgulas finais de um JSONC.
fn strip_json_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_str = false;
    while let Some(c) = chars.next() {
        if in_str {
            out.push(c);
            if c == '\\' {
                if let Some(n) = chars.next() {
                    out.push(n);
                }
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        match (c, chars.peek()) {
            ('"', _) => {
                in_str = true;
                out.push(c);
            }
            ('/', Some('/')) => {
                for n in chars.by_ref() {
                    if n == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            }
            ('/', Some('*')) => {
                chars.next();
                let mut prev = ' ';
                for n in chars.by_ref() {
                    if prev == '*' && n == '/' {
                        break;
                    }
                    prev = n;
                }
            }
            _ => out.push(c),
        }
    }
    // Vírgula antes de `}` ou `]`.
    let mut clean = String::with_capacity(out.len());
    let bytes: Vec<char> = out.chars().collect();
    let mut in_str = false;
    for (i, &c) in bytes.iter().enumerate() {
        if c == '"' && (i == 0 || bytes[i - 1] != '\\') {
            in_str = !in_str;
        }
        if !in_str && c == ',' {
            let next = bytes[i + 1..].iter().find(|c| !c.is_whitespace());
            if matches!(next, Some('}') | Some(']')) {
                continue;
            }
        }
        clean.push(c);
    }
    clean
}

/// Ferramentas nativas da CLI do Grok bloqueadas (só o MCP do Treasury fica).
const GROK_DENIED: &[&str] = &[
    "Bash",
    "Edit",
    "Write",
    "Read",
    "Grep",
    "Glob",
    "WebFetch",
    "WebSearch",
];
/// Liberadas quando o usuário permite olhar o computador nesta conversa.
const GROK_READ_TOOLS: &[&str] = &["Read", "Grep", "Glob"];

/// Nomes dos MCPs que a CLI do Grok carrega para o usuário (config, plugins e importados),
/// só para negar as ferramentas deles nesta execução: `~/.grok/config.toml` e a lista do
/// `grok inspect` na pasta de trabalho (melhor esforço, até 5 s). Nada é alterado.
pub fn grok_loaded_mcps(home: Option<&Path>, workdir: &Path) -> Vec<String> {
    let mut names = grok_config_mcps(home);
    for n in grok_inspect_mcps(workdir) {
        if !names.contains(&n) {
            names.push(n);
        }
    }
    names
}

/// Servidores listados em "MCP Servers" no `grok inspect` (linhas `└ nome (stdio) …`).
fn grok_inspect_mcps(workdir: &Path) -> Vec<String> {
    let Some(exe) = find_executable("grok") else {
        return vec![];
    };
    let mut cmd = Command::new(exe);
    cmd.arg("inspect")
        .current_dir(workdir)
        .env("PATH", search_path())
        .envs(cli_env(Provider::GrokCli).iter().copied())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let Ok(child) = cmd.spawn() else {
        return vec![];
    };
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });
    let Ok(Ok(out)) = rx.recv_timeout(std::time::Duration::from_secs(5)) else {
        return vec![];
    };
    parse_inspect_mcps(&String::from_utf8_lossy(&out.stdout))
}

fn parse_inspect_mcps(text: &str) -> Vec<String> {
    let mut inside = false;
    let mut out = vec![];
    for line in text.lines() {
        let t = line.trim_start();
        if t.starts_with("MCP Servers") {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        let Some(rest) = t.strip_prefix('└') else {
            if !t.is_empty() {
                break;
            }
            continue;
        };
        if let Some(name) = rest.split_whitespace().next() {
            out.push(name.to_string());
        }
    }
    out
}

/// Chave TOML sem aspas (letras, números, `_` e `-`).
fn is_plain_key(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// `[mcp_servers.<nome>]` em ~/.codex/config.toml (ou `$CODEX_HOME`), só para desligá-los
/// por `-c` nesta execução. O arquivo não é alterado.
pub fn codex_config_mcps(home: Option<&Path>) -> Vec<String> {
    let base = std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| home.map(|h| h.join(".codex")));
    toml_mcp_names(base.map(|b| b.join("config.toml")))
}

fn toml_mcp_names(path: Option<PathBuf>) -> Vec<String> {
    let Some(text) = path.and_then(|p| std::fs::read_to_string(p).ok()) else {
        return vec![];
    };
    match text.parse::<toml::Table>() {
        Ok(t) => t
            .get("mcp_servers")
            .and_then(|v| v.as_table())
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default(),
        Err(_) => vec![],
    }
}

/// MCPs do usuário no Claude Code (`mcpServers` de `~/.claude.json`), com a definição.
pub fn claude_config_mcps(home: Option<&Path>) -> serde_json::Map<String, Value> {
    home.and_then(|h| std::fs::read_to_string(h.join(".claude.json")).ok())
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| v.get("mcpServers").and_then(Value::as_object).cloned())
        .unwrap_or_default()
}

/// MCPs do usuário no Gemini CLI (`mcpServers` de `~/.gemini/settings.json`).
pub fn gemini_config_mcps(home: Option<&Path>) -> Vec<String> {
    home.and_then(|h| std::fs::read_to_string(h.join(".gemini").join("settings.json")).ok())
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| {
            v.get("mcpServers")
                .and_then(Value::as_object)
                .map(|m| m.keys().cloned().collect())
        })
        .unwrap_or_default()
}

/// Os MCPs que cada CLI instalada carregaria por conta própria (fora o Treasury), para a
/// pessoa escolher quais o agente pode usar. O do Grok inclui os que ele importa do Claude e
/// do Cursor (`grok inspect`, até 5 s).
pub fn user_mcps_by_cli(home: Option<&Path>, grok_workdir: &Path) -> Vec<(Provider, Vec<String>)> {
    let mut out = vec![];
    for p in ALL_PROVIDERS {
        if p.binary().and_then(find_executable).is_none() {
            continue;
        }
        let mut names: Vec<String> = match p {
            Provider::ClaudeCode => claude_config_mcps(home).keys().cloned().collect(),
            Provider::Codex => codex_config_mcps(home),
            Provider::GrokCli => grok_loaded_mcps(home, grok_workdir),
            Provider::OpencodeCli => opencode_config_mcps(home),
            Provider::GeminiCli => gemini_config_mcps(home),
            _ => continue,
        };
        names.retain(|n| n != crate::mcp::SERVER_NAME);
        names.sort();
        names.dedup();
        out.push((*p, names));
    }
    out
}

/// `[mcp_servers.<nome>]` em ~/.grok/config.toml (ou `$GROK_HOME`).
pub fn grok_config_mcps(home: Option<&Path>) -> Vec<String> {
    let base = std::env::var_os("GROK_HOME")
        .map(PathBuf::from)
        .or_else(|| home.map(|h| h.join(".grok")));
    toml_mcp_names(base.map(|b| b.join("config.toml")))
}

/// Variáveis de ambiente da execução por CLI: no Grok, sem importar os MCPs do Claude e do
/// Cursor do usuário.
pub fn cli_env(provider: Provider) -> &'static [(&'static str, &'static str)] {
    match provider {
        Provider::GrokCli => &[
            ("GROK_CLAUDE_MCPS_ENABLED", "false"),
            ("GROK_CURSOR_MCPS_ENABLED", "false"),
        ],
        _ => &[],
    }
}

/// Variáveis de ambiente que dependem da configuração desta execução: no OpenCode, a
/// configuração gerada por [`write_mcp_config`].
pub fn config_env(provider: Provider, mcp_config: &str) -> Vec<(&'static str, String)> {
    match provider {
        Provider::OpencodeCli => vec![("OPENCODE_CONFIG_CONTENT", mcp_config.to_string())],
        _ => vec![],
    }
}

/// Gera os arquivos de configuração e devolve o valor de `{mcp_config}`.
pub fn write_mcp_config(
    provider: Provider,
    t: &McpTarget<'_>,
    private_dir: &Path,
    workdir: &Path,
    policy: &CliPolicy,
) -> Result<String> {
    let server = server_json(t);
    match provider {
        // TOML em linha para `-c mcp_servers.treasury=…`.
        Provider::Codex => {
            let env = server["env"]
                .as_object()
                .map(|m| {
                    m.iter()
                        .map(|(k, v)| format!("{k}={}", toml_str(v.as_str().unwrap_or(""))))
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .unwrap_or_default();
            Ok(format!(
                "{{command={},args=[\"mcp\"],env={{{env}}}}}",
                toml_str(&t.command.to_string_lossy())
            ))
        }
        // Configuração do projeto em `{workdir}/.gemini/settings.json`.
        Provider::GeminiCli => {
            let dir = workdir.join(".gemini");
            crate::paths::create_private_dir(&dir)?;
            let mut s = server.clone();
            s["trust"] = json!(true);
            let excluded = policy.gemini_excluded();
            let cfg = json!({
                "mcpServers": {crate::mcp::SERVER_NAME: s},
                "excludeTools": excluded,
                "tools": {"exclude": excluded},
            });
            let path = dir.join("settings.json");
            write_private(&path, &serde_json::to_vec_pretty(&cfg)?)?;
            Ok(path.to_string_lossy().into_owned())
        }
        // Grok: servidor do projeto em `{workdir}/.grok/config.toml` (a CLI não aceita um
        // arquivo de MCP por parâmetro).
        Provider::GrokCli => {
            let dir = workdir.join(".grok");
            crate::paths::create_private_dir(&dir)?;
            let env = server["env"]
                .as_object()
                .map(|m| {
                    m.iter()
                        .map(|(k, v)| format!("{k} = {}", toml_str(v.as_str().unwrap_or(""))))
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            let mut cfg = format!(
                "# Gerado pelo Treasury a cada conversa; só o MCP do Treasury.\n[mcp_servers.{name}]\ncommand = {cmd}\nargs = [\"mcp\"]\nenv = {{ {env} }}\nenabled = true\n",
                name = crate::mcp::SERVER_NAME,
                cmd = toml_str(&t.command.to_string_lossy()),
            );
            // O servidor de mesmo nome na pasta do projeto substitui o do usuário, do Claude e
            // do Cursor: desligado aqui, nem sobe (as regras `--deny` só barravam as chamadas).
            for name in policy
                .user_mcps
                .iter()
                .filter(|n| *n != crate::mcp::SERVER_NAME && !policy.allowed(n))
            {
                cfg.push_str(&format!(
                    "\n[mcp_servers.{}]\ncommand = \"true\"\nenabled = false\n",
                    toml_str(name)
                ));
            }
            let path = dir.join("config.toml");
            write_private(&path, cfg.as_bytes())?;
            Ok(path.to_string_lossy().into_owned())
        }
        // OpenCode: a configuração inteira, que vai na variável `OPENCODE_CONFIG_CONTENT`
        // (ver [`config_env`]), e não na linha de comando.
        Provider::OpencodeCli => Ok(opencode_config(&server, policy).to_string()),
        // Formato padrão `mcpServers` (Claude Code e personalizadas).
        _ => {
            let mut servers = if provider == Provider::ClaudeCode {
                policy.claude_user_mcps.clone()
            } else {
                serde_json::Map::new()
            };
            servers.insert(crate::mcp::SERVER_NAME.into(), server);
            let cfg = json!({"mcpServers": servers});
            let path = private_dir.join("mcp.json");
            write_private(&path, &serde_json::to_vec_pretty(&cfg)?)?;
            Ok(path.to_string_lossy().into_owned())
        }
    }
}

pub fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    let _ = std::fs::remove_file(path);
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts.open(path)?;
    std::io::Write::write_all(&mut f, bytes)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Execução

/// O que sai da CLI, já interpretado.
#[derive(Debug, Clone, PartialEq)]
pub enum CliEvent {
    Text(String),
    /// Raciocínio do modelo (blocos `thinking`).
    Reasoning(String),
    /// Ferramenta nativa da CLI chamada (as do Treasury são informadas pelo MCP).
    ToolUse {
        id: String,
        name: String,
        args: String,
    },
    /// Resultado de uma ferramenta nativa.
    ToolResult {
        id: String,
        is_error: bool,
        summary: String,
    },
    Usage(Tokens),
    /// Resultado final (Claude Code: `result`).
    Final {
        text: String,
        is_error: bool,
    },
}

/// Ferramentas do Treasury vistas pela CLI (`mcp__treasury__<nome>`): o MCP já as informa.
const TREASURY_TOOL_PREFIX: &str = "mcp__treasury__";

fn short(s: &str, max: usize) -> String {
    let flat = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() > max {
        format!("{}…", flat.chars().take(max - 1).collect::<String>())
    } else {
        flat
    }
}

/// Interpreta uma linha do `stream-json` do Claude Code.
pub fn parse_stream_json(line: &str) -> Vec<CliEvent> {
    let Ok(v) = serde_json::from_str::<Value>(line) else {
        return vec![];
    };
    match v["type"].as_str() {
        Some("assistant") => v["message"]["content"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|b| match b["type"].as_str() {
                Some("text") => b["text"]
                    .as_str()
                    .filter(|t| !t.is_empty())
                    .map(|t| CliEvent::Text(t.to_string())),
                Some("thinking") => b["thinking"]
                    .as_str()
                    .filter(|t| !t.trim().is_empty())
                    .map(|t| CliEvent::Reasoning(t.to_string())),
                Some("tool_use") => {
                    let name = b["name"].as_str().unwrap_or_default();
                    (!name.starts_with(TREASURY_TOOL_PREFIX) && !name.is_empty()).then(|| {
                        CliEvent::ToolUse {
                            id: b["id"].as_str().unwrap_or_default().to_string(),
                            name: name.to_string(),
                            args: short(&b["input"].to_string(), 200),
                        }
                    })
                }
                _ => None,
            })
            .collect(),
        // Resultados de ferramentas voltam como mensagem do "usuário" no stream.
        Some("user") => v["message"]["content"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|b| b["type"] == "tool_result")
            .map(|b| {
                let is_error = b["is_error"].as_bool().unwrap_or(false);
                CliEvent::ToolResult {
                    id: b["tool_use_id"].as_str().unwrap_or_default().to_string(),
                    is_error,
                    // Só o erro (curto): o conteúdo lido de arquivos não vai para a conversa.
                    summary: if is_error {
                        let text = b["content"]
                            .as_str()
                            .map(str::to_string)
                            .unwrap_or_else(|| b["content"].to_string());
                        short(&text, 200)
                    } else {
                        i18n::t("nucleo-ia-cli-concluida")
                    },
                }
            })
            .collect(),
        Some("result") => {
            let mut out = vec![];
            let u = &v["usage"];
            if u.is_object() {
                let cached = u["cache_read_input_tokens"].as_i64().unwrap_or(0);
                let written = u["cache_creation_input_tokens"].as_i64().unwrap_or(0);
                out.push(CliEvent::Usage(Tokens {
                    input: u["input_tokens"].as_i64().unwrap_or(0) + cached + written,
                    output: u["output_tokens"].as_i64().unwrap_or(0),
                    cached,
                    cache_write: written,
                }));
            }
            out.push(CliEvent::Final {
                text: v["result"].as_str().unwrap_or("").to_string(),
                is_error: v["is_error"].as_bool().unwrap_or(false)
                    || v["subtype"].as_str().is_some_and(|s| s != "success"),
            });
            out
        }
        _ => vec![],
    }
}

/// Interpreta uma linha de `opencode run --format json` (um evento por linha: `text`,
/// `reasoning`, `tool_use`, `step_finish` com os tokens, `error`). As ferramentas do
/// Treasury (e o `execute` da 2.x, que só as chama) já são informadas pelo MCP.
pub fn parse_opencode_json(line: &str) -> Vec<CliEvent> {
    let Ok(v) = serde_json::from_str::<Value>(line) else {
        return vec![];
    };
    let part = &v["part"];
    match v["type"].as_str() {
        Some("text") => part["text"]
            .as_str()
            .filter(|t| !t.is_empty())
            .map(|t| vec![CliEvent::Text(t.to_string())])
            .unwrap_or_default(),
        Some("reasoning") => part["text"]
            .as_str()
            .filter(|t| !t.trim().is_empty())
            .map(|t| vec![CliEvent::Reasoning(t.to_string())])
            .unwrap_or_default(),
        Some("tool_use") => {
            let name = part["tool"].as_str().unwrap_or_default();
            if name.is_empty() || name == "execute" || name.starts_with(crate::mcp::SERVER_NAME) {
                return vec![];
            }
            let id = part["callID"]
                .as_str()
                .or(part["id"].as_str())
                .unwrap_or_default()
                .to_string();
            let state = &part["state"];
            let is_error = state["status"] == "error";
            vec![
                CliEvent::ToolUse {
                    id: id.clone(),
                    name: name.to_string(),
                    args: short(&state["input"].to_string(), 200),
                },
                CliEvent::ToolResult {
                    id,
                    is_error,
                    // Só o erro: o conteúdo lido de arquivos não vai para a conversa.
                    summary: if is_error {
                        short(
                            state["error"]
                                .as_str()
                                .map(str::to_string)
                                .unwrap_or_else(|| state["error"].to_string())
                                .as_str(),
                            200,
                        )
                    } else {
                        i18n::t("nucleo-ia-cli-concluida")
                    },
                },
            ]
        }
        Some("step_finish") => {
            let t = &part["tokens"];
            if !t.is_object() {
                return vec![];
            }
            let n = |v: &Value| v.as_i64().unwrap_or(0);
            let cached = n(&t["cache"]["read"]);
            let written = n(&t["cache"]["write"]);
            vec![CliEvent::Usage(Tokens {
                input: n(&t["input"]) + cached + written,
                output: n(&t["output"]) + n(&t["reasoning"]),
                cached,
                cache_write: written,
            })]
        }
        Some("error") => {
            let e = &v["error"];
            let text = e["message"]
                .as_str()
                .or(e["data"]["message"].as_str())
                .or(e["name"].as_str())
                .map_or_else(|| i18n::t("nucleo-ia-cli-opencode-erro"), str::to_string);
            vec![CliEvent::Final {
                text,
                is_error: true,
            }]
        }
        _ => vec![],
    }
}

/// O que a execução entrega ao job, na ordem.
#[derive(Debug, Clone, PartialEq)]
pub enum CliStep<'a> {
    Text(&'a str),
    Reasoning(&'a str),
    /// Ferramenta nativa da CLI (nome com o prefixo `cli.`).
    ToolCall {
        name: &'a str,
        args: &'a str,
    },
    ToolResult {
        name: &'a str,
        ok: bool,
        summary: &'a str,
    },
}

pub struct CliRun<'a> {
    pub engine: &'a Engine,
    pub argv: Vec<String>,
    pub workdir: &'a Path,
    pub cancelled: &'a (dyn Fn() -> bool + Sync),
    /// Entrada padrão (Claude Code com `--input-format stream-json`); sem isso, vazia.
    pub stdin: Option<Vec<u8>>,
    /// Variáveis de ambiente desta execução ([`config_env`]).
    pub env: Vec<(&'static str, String)>,
    /// Tempo-limite desta execução.
    pub timeout: Duration,
}

/// Mensagem do usuário no formato `stream-json` de entrada do Claude Code: as imagens em
/// base64 (blocos `image`, como na API da Anthropic) e o prompt.
pub fn stream_json_user_message(prompt: &str, images: &[super::agent::CliImage]) -> Vec<u8> {
    let mut content: Vec<Value> = images
        .iter()
        .map(|i| {
            json!({"type": "image", "source": {
                "type": "base64", "media_type": i.mime, "data": super::attachments::b64(&i.data),
            }})
        })
        .collect();
    content.push(json!({"type": "text", "text": prompt}));
    let mut line = serde_json::to_vec(&json!({
        "type": "user",
        "message": {"role": "user", "content": content},
    }))
    .expect("JSON");
    line.push(b'\n');
    line
}

/// Texto que fica no lugar do prompt na linha de comando do Gemini CLI (o pedido vai pela
/// entrada padrão; o `-p` é acrescentado depois dela).
/// Codex sem acesso ao computador: sem o shell (que lê o disco mesmo no sandbox somente
/// leitura) e sem a ferramenta que abre imagens do disco.
pub const CODEX_NO_DISK: &[&str] = &["features.shell_tool=false", "tools.view_image=false"];

pub const GEMINI_STDIN_PROMPT: &str = "Answer the request above.";

/// Leva o prompt (e as imagens do pedido) à CLI **fora da linha de comando**: argumentos
/// aparecem em `ps` para outros processos do computador e têm limite de tamanho
/// (`ARG_MAX`), e o prompt leva dados financeiros (assessment S1).
///
/// - **Claude Code**: o prompt sai da linha de comando e vai pela entrada padrão; com
///   `--output-format stream-json` (o template padrão), numa mensagem `stream-json`
///   (`--input-format stream-json`) com as imagens em base64; nada vai para o disco.
/// - **Codex**: `-` no lugar do prompt (`codex exec` lê as instruções da entrada padrão).
///   `codex exec --image <arquivo>` só aceita arquivos: as imagens são gravadas `0600` no
///   diretório privado do job (`~/.treasury/run/agents/<job>`, `0700`), apagado assim que a
///   CLI termina (ou é cancelada).
/// - **Gemini CLI**: o pedido pela entrada padrão e, no `-p`, só [`GEMINI_STDIN_PROMPT`]
///   (o Gemini acrescenta o `-p` depois do que leu da entrada).
/// - **Grok**: `--prompt-file` com o prompt num arquivo `0600` do diretório privado do job.
/// - **OpenCode**: pela entrada padrão (`opencode run` sem mensagem lê dela).
/// - **CLI personalizada**: o template é do usuário; o prompt vai onde ele pôs `{prompt}`.
///
/// Devolve o que mandar pela entrada padrão.
pub fn send_prompt(
    provider: Provider,
    argv: &mut Vec<String>,
    prompt: &str,
    images: &[super::agent::CliImage],
    private_dir: &Path,
) -> std::result::Result<Option<Vec<u8>>, String> {
    let pos = argv.iter().skip(1).position(|a| a == prompt).map(|i| i + 1);
    match provider {
        Provider::ClaudeCode => {
            let at = pos.ok_or_else(|| prompt_argument("Claude Code"))?;
            let streams = argv
                .windows(2)
                .any(|w| w[0] == "--output-format" && w[1] == "stream-json");
            if !images.is_empty() && !streams {
                return Err(i18n::t("nucleo-ia-cli-claude-stream"));
            }
            argv.remove(at);
            argv.retain(|a| !a.starts_with("--input-format"));
            if streams {
                let at = 1.min(argv.len());
                argv.insert(at, "stream-json".into());
                argv.insert(at, "--input-format".into());
                Ok(Some(stream_json_user_message(prompt, images)))
            } else {
                Ok(Some(prompt.as_bytes().to_vec()))
            }
        }
        Provider::Codex => {
            let at = pos.ok_or_else(|| prompt_argument("Codex"))?;
            argv[at] = "-".into();
            let at = argv.iter().position(|a| a == "exec").map_or(1, |i| i + 1);
            let at = at.min(argv.len());
            for (i, img) in images.iter().enumerate().rev() {
                let ext = img.mime.rsplit('/').next().unwrap_or("png");
                let path = private_dir.join(format!("anexo-{}.{ext}", i + 1));
                write_private(&path, &img.data).map_err(|e| e.to_string())?;
                // `--image=<arquivo>`: com `=`, o argumento leva um valor só.
                argv.insert(at, format!("--image={}", path.to_string_lossy()));
            }
            Ok(Some(prompt.as_bytes().to_vec()))
        }
        Provider::GeminiCli => {
            if !images.is_empty() {
                return Err(no_images(provider));
            }
            let at = pos.ok_or_else(|| prompt_argument("Gemini CLI"))?;
            argv[at] = GEMINI_STDIN_PROMPT.into();
            let mut bytes = prompt.as_bytes().to_vec();
            bytes.push(b'\n');
            Ok(Some(bytes))
        }
        Provider::GrokCli => {
            if !images.is_empty() {
                return Err(no_images(provider));
            }
            let at = pos.ok_or_else(|| prompt_argument("Grok"))?;
            let path = private_dir.join("prompt.txt");
            write_private(&path, prompt.as_bytes()).map_err(|e| e.to_string())?;
            // `-p <prompt>` vira `--prompt-file <arquivo>`.
            if at >= 2 && matches!(argv[at - 1].as_str(), "-p" | "--single") {
                argv[at - 1] = "--prompt-file".into();
                argv[at] = path.to_string_lossy().into_owned();
            } else {
                argv.remove(at);
                argv.push("--prompt-file".into());
                argv.push(path.to_string_lossy().into_owned());
            }
            Ok(None)
        }
        Provider::OpencodeCli => {
            if !images.is_empty() {
                return Err(no_images(provider));
            }
            let at = pos.ok_or_else(|| prompt_argument("OpenCode"))?;
            argv.remove(at);
            Ok(Some(prompt.as_bytes().to_vec()))
        }
        _ if !images.is_empty() => Err(no_images(provider)),
        _ => Ok(None),
    }
}

/// O template da CLI sem `{prompt}` num argumento só dele.
fn prompt_argument(cli: &str) -> String {
    i18n::t_args(
        "nucleo-ia-cli-prompt-argumento",
        &i18n::args(&[("cli", cli.to_string().into())]),
    )
}

/// A CLI não lê as imagens anexadas.
fn no_images(provider: Provider) -> String {
    i18n::t_args(
        "nucleo-ia-cli-sem-imagens",
        &i18n::args(&[("motor", provider.label().into())]),
    )
}

#[derive(Debug, Default)]
pub struct CliOutput {
    pub text: String,
    pub tokens: Tokens,
    pub usage_reported: bool,
    /// Passou do tempo-limite: `text` é o que a CLI escreveu até ali (o que ela fez pelas
    /// ferramentas do Treasury já está gravado ou na fila).
    pub timed_out: bool,
}

/// Encerra a CLI e os processos que ela abriu (o grupo dela, no Unix).
fn kill_tree(child: &mut std::process::Child) {
    #[cfg(unix)]
    if let Ok(pid) = i32::try_from(child.id()) {
        // SAFETY: `kill` com o id negativo manda o sinal ao grupo que a CLI lidera.
        unsafe {
            libc::kill(-pid, libc::SIGKILL);
        }
    }
    let _ = child.kill();
    let _ = child.wait();
}

/// Executa a CLI, entrega texto, raciocínio e ferramentas nativas a `on_step` e devolve o
/// texto final.
pub fn run(
    r: &CliRun<'_>,
    on_step: &mut dyn FnMut(CliStep<'_>),
) -> std::result::Result<CliOutput, String> {
    let program = &r.argv[0];
    let exe = find_executable(program).ok_or_else(|| {
        i18n::t_args(
            "nucleo-ia-cli-nao-encontrada",
            &i18n::args(&[("programa", program.clone().into())]),
        )
    })?;
    let mut cmd = Command::new(&exe);
    // Grupo de processos próprio: no tempo-limite ou no cancelamento, os filhos da CLI (os
    // servidores MCP que ela abriu, um `sleep` de script) saem junto e não seguram a saída.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut child = cmd
        .args(&r.argv[1..])
        .current_dir(r.workdir)
        .env("PATH", search_path())
        .envs(cli_env(r.engine.provider).iter().copied())
        .envs(r.env.iter().map(|(k, v)| (*k, v.as_str())))
        .stdin(if r.stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            i18n::t_args(
                "nucleo-ia-cli-nao-iniciou",
                &i18n::args(&[
                    ("programa", program.clone().into()),
                    ("detalhe", e.to_string().into()),
                ]),
            )
        })?;
    // Entrada padrão numa thread (pode ser grande) e fechada no fim: a CLI sabe que acabou.
    let writer = match (r.stdin.clone(), child.stdin.take()) {
        (Some(bytes), Some(mut pipe)) => Some(std::thread::spawn(move || {
            let _ = std::io::Write::write_all(&mut pipe, &bytes);
        })),
        _ => None,
    };
    let (tx, rx) = mpsc::channel::<String>();
    let stdout = child.stdout.take().expect("stdout");
    let reader = std::thread::spawn(move || {
        let rd = BufReader::new(stdout);
        for line in rd.lines() {
            let Ok(line) = line else { break };
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    let mut stderr = child.stderr.take().expect("stderr");
    let err_reader = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = (&mut stderr).take(64 * 1024).read_to_string(&mut s);
        s
    });
    let protocol = r.engine.provider.protocol();
    let stream_json = matches!(
        protocol,
        Protocol::CliStreamJson | Protocol::CliOpencodeJson
    );
    // O OpenCode informa os tokens por passo; o Claude Code, o total no fim.
    let per_step_usage = protocol == Protocol::CliOpencodeJson;
    let started = Instant::now();
    let mut out = CliOutput::default();
    let mut final_text: Option<(String, bool)> = None;
    // Ferramentas nativas abertas (id → nome mostrado).
    let mut open_tools: HashMap<String, String> = HashMap::new();
    let mut handle = |line: String, out: &mut CliOutput| {
        if stream_json {
            let events = if per_step_usage {
                parse_opencode_json(&line)
            } else {
                parse_stream_json(&line)
            };
            for ev in events {
                match ev {
                    CliEvent::Text(t) => {
                        if !out.text.is_empty() && !out.text.ends_with('\n') {
                            out.text.push_str("\n\n");
                            on_step(CliStep::Text("\n\n"));
                        }
                        out.text.push_str(&t);
                        on_step(CliStep::Text(&t));
                    }
                    CliEvent::Reasoning(t) => on_step(CliStep::Reasoning(&t)),
                    CliEvent::ToolUse { id, name, args } => {
                        let shown = format!("cli.{name}");
                        on_step(CliStep::ToolCall {
                            name: &shown,
                            args: &args,
                        });
                        open_tools.insert(id, shown);
                    }
                    CliEvent::ToolResult {
                        id,
                        is_error,
                        summary,
                    } => {
                        if let Some(name) = open_tools.remove(&id) {
                            on_step(CliStep::ToolResult {
                                name: &name,
                                ok: !is_error,
                                summary: &summary,
                            });
                        }
                    }
                    CliEvent::Usage(t) => {
                        if per_step_usage {
                            out.tokens.add(t);
                        } else {
                            out.tokens = t;
                        }
                        out.usage_reported = true;
                    }
                    CliEvent::Final { text, is_error } => final_text = Some((text, is_error)),
                }
            }
        } else {
            let t = format!("{line}\n");
            out.text.push_str(&t);
            on_step(CliStep::Text(&t));
        }
    };
    let status = loop {
        if (r.cancelled)() {
            kill_tree(&mut child);
            let _ = reader.join();
            return Err(i18n::t("nucleo-ia-agente-cancelado"));
        }
        // Tempo-limite: encerra a CLI mas devolve o que ela já escreveu (a conversa guarda a
        // parte feita, com as ferramentas usadas), em vez de perder tudo.
        if started.elapsed() > r.timeout {
            kill_tree(&mut child);
            let _ = reader.join();
            for line in rx.try_iter() {
                handle(line, &mut out);
            }
            out.timed_out = true;
            if !out.usage_reported {
                out.tokens.output = super::usage::approx_tokens(out.text.len());
            }
            return Ok(out);
        }
        match rx.recv_timeout(Duration::from_millis(150)) {
            Ok(line) => handle(line, &mut out),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                break child.wait().map_err(|e| e.to_string())?;
            }
        }
    };
    let _ = reader.join();
    if let Some(w) = writer {
        let _ = w.join();
    }
    for line in rx.try_iter() {
        handle(line, &mut out);
    }
    let err_text = err_reader.join().unwrap_or_default();
    if let Some((text, is_error)) = final_text {
        if is_error {
            return Err(if text.is_empty() {
                i18n::t("nucleo-ia-cli-erro")
            } else {
                text.chars().take(500).collect()
            });
        }
        if out.text.trim().is_empty() && !text.is_empty() {
            on_step(CliStep::Text(&text));
            out.text = text;
        }
    }
    if !status.success() && out.text.trim().is_empty() {
        let tail: String = err_text
            .lines()
            .rev()
            .take(6)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        let tail = if tail.is_empty() {
            String::new()
        } else {
            format!(": {}", tail.chars().take(500).collect::<String>())
        };
        return Err(i18n::t_args(
            "nucleo-ia-cli-terminou",
            &i18n::args(&[
                ("programa", program.clone().into()),
                ("status", status.to_string().into()),
                ("saida", tail.into()),
            ]),
        ));
    }
    if !out.usage_reported {
        out.tokens.output = super::usage::approx_tokens(out.text.len());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(model: &str) -> HashMap<&'static str, String> {
        HashMap::from([
            ("prompt", "Qual o saldo? \"aspas\" {model}".to_string()),
            ("mcp_config", "/tmp/x/mcp.json".to_string()),
            ("model", model.to_string()),
            ("workdir", "/tmp/w".to_string()),
        ])
    }

    #[test]
    fn expands_without_shell_and_drops_empty_groups() {
        let argv = expand(super::super::engine::CLAUDE_CODE_TEMPLATE, &vars("")).unwrap();
        assert_eq!(argv[0], "claude");
        assert_eq!(argv[1], "-p");
        assert_eq!(
            argv[2], "Qual o saldo? \"aspas\" {model}",
            "prompt é um argumento só e não é expandido de novo"
        );
        assert!(!argv.contains(&"--model".to_string()));
        assert!(argv.contains(&"mcp__treasury__*".to_string()));
        assert!(argv.contains(&"--strict-mcp-config".to_string()));
        let argv = expand(super::super::engine::CLAUDE_CODE_TEMPLATE, &vars("sonnet")).unwrap();
        let i = argv.iter().position(|a| a == "--model").unwrap();
        assert_eq!(argv[i + 1], "sonnet");
        let codex = expand(super::super::engine::CODEX_TEMPLATE, &vars("")).unwrap();
        assert!(codex.contains(&"read-only".to_string()));
        assert!(codex.contains(&"mcp_servers.treasury=/tmp/x/mcp.json".to_string()));
        assert!(split_template("a \"b").is_err());
        assert!(split_template("a [b").is_err());
    }

    #[test]
    fn parses_claude_stream_json() {
        let a = parse_stream_json(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Olá"},{"type":"tool_use","name":"mcp__treasury__get_dashboard","input":{}}]}}"#,
        );
        assert_eq!(a, vec![CliEvent::Text("Olá".into())]);
        let r = parse_stream_json(
            r#"{"type":"result","subtype":"success","is_error":false,"result":"Fim","usage":{"input_tokens":10,"output_tokens":5}}"#,
        );
        assert_eq!(
            r,
            vec![
                CliEvent::Usage(Tokens {
                    input: 10,
                    output: 5,
                    cached: 0,
                    cache_write: 0
                }),
                CliEvent::Final {
                    text: "Fim".into(),
                    is_error: false
                }
            ]
        );
        assert!(parse_stream_json("não é json").is_empty());
    }

    #[test]
    fn codex_inline_toml_escapes() {
        let dir = tempfile::tempdir().unwrap();
        let t = McpTarget {
            command: Path::new("/Apps/Treasury \"x\"/treasury"),
            home: None,
            token_file: Path::new("/r/t"),
        };
        let off = CliPolicy::default();
        let v = write_mcp_config(Provider::Codex, &t, dir.path(), dir.path(), &off).unwrap();
        assert!(v.starts_with("{command=\"/Apps/Treasury \\\"x\\\"/treasury\""));
        assert!(v.contains("TREASURY_MCP_TOKEN_FILE=\"/r/t\""));
        let p = write_mcp_config(Provider::ClaudeCode, &t, dir.path(), dir.path(), &off).unwrap();
        let cfg: Value = serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
        assert_eq!(cfg["mcpServers"]["treasury"]["args"], json!(["mcp"]));
        let g = write_mcp_config(Provider::GeminiCli, &t, dir.path(), dir.path(), &off).unwrap();
        assert!(Path::new(&g).ends_with(".gemini/settings.json"));
        // Sem acesso ao computador, a Gemini CLI não lê arquivos; com acesso, só lê.
        let cfg: Value = serde_json::from_slice(&std::fs::read(&g).unwrap()).unwrap();
        let excluded: Vec<&str> = cfg["excludeTools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(excluded.contains(&"read_file") && excluded.contains(&"run_shell_command"));
        let on = CliPolicy {
            computer_access: true,
            home: None,
            ..Default::default()
        };
        let g = write_mcp_config(Provider::GeminiCli, &t, dir.path(), dir.path(), &on).unwrap();
        let cfg: Value = serde_json::from_slice(&std::fs::read(&g).unwrap()).unwrap();
        let excluded = cfg["excludeTools"].to_string();
        assert!(!excluded.contains("read_file"), "{excluded}");
        assert!(excluded.contains("run_shell_command") && excluded.contains("write_file"));
    }

    fn argv(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn value_after<'a>(args: &'a [String], flag: &str) -> Vec<&'a str> {
        let Some(i) = args.iter().position(|a| a == flag) else {
            return vec![];
        };
        args[i + 1..]
            .iter()
            .take_while(|a| !a.starts_with('-'))
            .map(String::as_str)
            .collect()
    }

    #[test]
    fn claude_code_gets_no_native_tools_even_with_an_edited_template() {
        let home = CliPolicy {
            computer_access: false,
            home: Some(PathBuf::from("/Users/ana")),
            ..Default::default()
        };
        // Template editado pelo usuário liberando tudo.
        let edited = argv(&[
            "claude",
            "-p",
            "Pedido",
            "--dangerously-skip-permissions",
            "--allowedTools",
            "Bash",
            "Read",
            "--permission-mode",
            "bypassPermissions",
            "--add-dir=/",
            "--tools",
            "default",
            "--output-format",
            "stream-json",
        ]);
        let out = restrict_argv(Provider::ClaudeCode, edited, &home);
        assert_eq!(&out[..3], &argv(&["claude", "-p", "Pedido"])[..]);
        for gone in [
            "--dangerously-skip-permissions",
            "bypassPermissions",
            "--add-dir=/",
            "default",
            "--permission-mode",
        ] {
            assert!(!out.contains(&gone.to_string()), "{gone} em {out:?}");
        }
        assert_eq!(value_after(&out, "--tools"), vec![""]);
        assert_eq!(
            value_after(&out, "--allowedTools"),
            vec!["mcp__treasury__*"]
        );
        assert!(value_after(&out, "--disallowedTools").contains(&"Bash"));
        assert!(out.contains(&"--strict-mcp-config".to_string()));
        assert!(out.contains(&"stream-json".to_string()));
        assert!(out.contains(&"--no-session-persistence".to_string()));
        assert!(!out.contains(&"--add-dir".to_string()));
        // O template padrão expandido continua igual no que importa.
        let std = expand(super::super::engine::CLAUDE_CODE_TEMPLATE, &vars("")).unwrap();
        let out = restrict_argv(Provider::ClaudeCode, std, &home);
        assert_eq!(out.iter().filter(|a| *a == "--tools").count(), 1);
        assert_eq!(value_after(&out, "--tools"), vec![""]);
        assert_eq!(out[2], "Qual o saldo? \"aspas\" {model}");
    }

    #[test]
    fn computer_access_opt_in_only_adds_read_tools() {
        let on = CliPolicy {
            computer_access: true,
            home: Some(PathBuf::from("/Users/ana")),
            ..Default::default()
        };
        let std = expand(super::super::engine::CLAUDE_CODE_TEMPLATE, &vars("")).unwrap();
        let out = restrict_argv(Provider::ClaudeCode, std, &on);
        assert_eq!(value_after(&out, "--tools"), vec!["Read,Glob,Grep"]);
        assert_eq!(
            value_after(&out, "--allowedTools"),
            vec!["mcp__treasury__*", "Read", "Glob", "Grep"]
        );
        assert_eq!(value_after(&out, "--add-dir"), vec!["/Users/ana"]);
        assert!(value_after(&out, "--disallowedTools").contains(&"Bash"));
        // Gemini: lê a pasta pessoal; Codex continua em somente leitura.
        let g = restrict_argv(
            Provider::GeminiCli,
            argv(&["gemini", "--yolo", "-p", "oi"]),
            &on,
        );
        // A lista de MCPs permitidos volta mesmo sem estar no template (só o Treasury).
        assert_eq!(
            g,
            argv(&[
                "gemini",
                "--include-directories",
                "/Users/ana",
                "--allowed-mcp-server-names",
                "treasury",
                "-p",
                "oi"
            ])
        );
        let c = restrict_argv(
            Provider::Codex,
            argv(&[
                "codex",
                "exec",
                "--dangerously-bypass-approvals-and-sandbox",
                "--sandbox",
                "danger-full-access",
                "oi",
            ]),
            &on,
        );
        assert_eq!(c, argv(&["codex", "exec", "--sandbox", "read-only", "oi"]));
        // Sem o acesso liberado, a Codex fica sem shell nem leitura de imagens do disco.
        let off = CliPolicy {
            computer_access: false,
            home: Some(PathBuf::from("/Users/ana")),
            ..Default::default()
        };
        let c = restrict_argv(Provider::Codex, argv(&["codex", "exec", "oi"]), &off);
        assert_eq!(
            c,
            argv(&[
                "codex",
                "-c",
                "features.shell_tool=false",
                "-c",
                "tools.view_image=false",
                "exec",
                "--sandbox",
                "read-only",
                "oi"
            ])
        );
        // CLIs desconhecidas: o template é do usuário.
        let custom = argv(&["minha-cli", "--yolo", "{prompt}"]);
        assert_eq!(
            restrict_argv(Provider::CustomCli, custom.clone(), &on),
            custom
        );
    }

    #[test]
    fn the_prompt_never_goes_on_the_command_line() {
        let dir = tempfile::tempdir().unwrap();
        let prompt = "Saldo da conta 1234: R$ 9.999,00";
        let none: &[super::super::agent::CliImage] = &[];
        // Claude Code com stream-json: mensagem de entrada pela entrada padrão.
        let mut a = argv(&["claude", "-p", prompt, "--output-format", "stream-json"]);
        let stdin = send_prompt(Provider::ClaudeCode, &mut a, prompt, none, dir.path())
            .unwrap()
            .unwrap();
        assert!(!a.iter().any(|x| x == prompt));
        assert_eq!(&a[1..3], &argv(&["--input-format", "stream-json"])[..]);
        assert!(String::from_utf8(stdin).unwrap().contains("R$ 9.999,00"));
        // Sem stream-json: o texto puro pela entrada padrão.
        let mut a = argv(&["claude", "-p", prompt]);
        let stdin = send_prompt(Provider::ClaudeCode, &mut a, prompt, none, dir.path())
            .unwrap()
            .unwrap();
        assert_eq!(a, argv(&["claude", "-p"]));
        assert_eq!(stdin, prompt.as_bytes());
        // Codex: `-` e a entrada padrão.
        let mut a = argv(&["codex", "exec", "--sandbox", "read-only", prompt]);
        let stdin = send_prompt(Provider::Codex, &mut a, prompt, none, dir.path())
            .unwrap()
            .unwrap();
        assert_eq!(a.last().unwrap(), "-");
        assert_eq!(stdin, prompt.as_bytes());
        // Gemini: o pedido pela entrada padrão e só um lembrete no -p.
        let mut a = argv(&["gemini", "-p", prompt]);
        let stdin = send_prompt(Provider::GeminiCli, &mut a, prompt, none, dir.path())
            .unwrap()
            .unwrap();
        assert_eq!(a, argv(&["gemini", "-p", GEMINI_STDIN_PROMPT]));
        assert!(stdin.starts_with(prompt.as_bytes()));
        // Grok: arquivo 0600 no diretório privado do job.
        let mut a = argv(&["grok", "--trust", "-p", prompt]);
        assert!(
            send_prompt(Provider::GrokCli, &mut a, prompt, none, dir.path())
                .unwrap()
                .is_none()
        );
        assert_eq!(&a[..3], &argv(&["grok", "--trust", "--prompt-file"])[..]);
        assert_eq!(std::fs::read_to_string(&a[3]).unwrap(), prompt);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&a[3]).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn stream_json_reasoning_and_native_tools() {
        let a = parse_stream_json(
            r#"{"type":"assistant","message":{"content":[{"type":"thinking","thinking":"Ver contas"},{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"/Users/ana/x.pdf"}},{"type":"tool_use","id":"t2","name":"mcp__treasury__list_accounts","input":{}}]}}"#,
        );
        assert_eq!(
            a,
            vec![
                CliEvent::Reasoning("Ver contas".into()),
                CliEvent::ToolUse {
                    id: "t1".into(),
                    name: "Read".into(),
                    args: "{\"file_path\":\"/Users/ana/x.pdf\"}".into(),
                },
            ]
        );
        let r = parse_stream_json(
            r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":"segredo do arquivo","is_error":false},{"type":"tool_result","tool_use_id":"t3","content":"Permission denied","is_error":true}]}}"#,
        );
        assert_eq!(
            r,
            vec![
                CliEvent::ToolResult {
                    id: "t1".into(),
                    is_error: false,
                    summary: "concluída".into(),
                },
                CliEvent::ToolResult {
                    id: "t3".into(),
                    is_error: true,
                    summary: "Permission denied".into(),
                },
            ]
        );
    }

    #[test]
    fn grok_gets_treasury_mcp_and_denies_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        let t = McpTarget {
            command: Path::new("/Applications/Treasury.app/Contents/MacOS/treasury"),
            home: None,
            token_file: Path::new("/tmp/x/token"),
        };
        let off = CliPolicy {
            user_mcps: vec!["github".into(), "treasury".into()],
            ..Default::default()
        };
        let p = write_mcp_config(Provider::GrokCli, &t, dir.path(), dir.path(), &off).unwrap();
        assert!(Path::new(&p).ends_with(".grok/config.toml"));
        let cfg: toml::Table = std::fs::read_to_string(&p).unwrap().parse().unwrap();
        let srv = &cfg["mcp_servers"]["treasury"];
        assert_eq!(srv["args"].as_array().unwrap()[0].as_str(), Some("mcp"));
        assert!(srv["env"].get(crate::mcp::TOKEN_FILE_ENV).is_some());

        let template = super::super::engine::GROK_CLI_TEMPLATE;
        let mut vars = HashMap::new();
        vars.insert("prompt", "oi".to_string());
        vars.insert("model", String::new());
        let mut argv0 = expand(template, &vars).unwrap();
        argv0.push("--always-approve".into());
        let out = restrict_argv(Provider::GrokCli, argv0, &off);
        assert!(out.contains(&"--trust".to_string()));
        assert!(!out.contains(&"--always-approve".to_string()));
        assert_eq!(value_after(&out, "--allow"), vec!["MCPTool(treasury__*)"]);
        let denied: Vec<&String> = out
            .windows(2)
            .filter(|w| w[0] == "--deny")
            .map(|w| &w[1])
            .collect();
        assert!(denied.iter().any(|d| *d == "Bash"));
        assert!(denied.iter().any(|d| *d == "MCPTool(github__*)"));
        assert!(!denied.iter().any(|d| d.contains("treasury")));
        // Com o computador liberado, a leitura volta.
        let on = CliPolicy {
            computer_access: true,
            ..Default::default()
        };
        let out = restrict_argv(Provider::GrokCli, argv(&["grok", "-p", "oi"]), &on);
        assert!(!out.windows(2).any(|w| w[0] == "--deny" && w[1] == "Read"));
        assert_eq!(cli_env(Provider::GrokCli).len(), 2);
    }

    #[test]
    fn user_mcps_stay_off_unless_the_user_allows_them() {
        let dir = tempfile::tempdir().unwrap();
        let t = McpTarget {
            command: Path::new("/Applications/Treasury.app/Contents/MacOS/treasury"),
            home: None,
            token_file: Path::new("/tmp/x/token"),
        };
        let policy = CliPolicy {
            user_mcps: vec!["github".into(), "rea".into(), "treasury".into()],
            allowed_mcps: vec!["rea".into()],
            ..Default::default()
        };
        // Grok: o não liberado fica desligado na pasta (nem sobe) e negado; o liberado, permitido.
        let p = write_mcp_config(Provider::GrokCli, &t, dir.path(), dir.path(), &policy).unwrap();
        let cfg: toml::Table = std::fs::read_to_string(&p).unwrap().parse().unwrap();
        assert_eq!(
            cfg["mcp_servers"]["github"]["enabled"].as_bool(),
            Some(false)
        );
        assert!(cfg["mcp_servers"].get("rea").is_none());
        assert_eq!(
            cfg["mcp_servers"]["treasury"]["enabled"].as_bool(),
            Some(true)
        );
        let out = restrict_argv(Provider::GrokCli, argv(&["grok", "-p", "oi"]), &policy);
        assert!(
            out.windows(2)
                .any(|w| w[0] == "--deny" && w[1] == "MCPTool(github__*)")
        );
        assert!(
            out.windows(2)
                .any(|w| w[0] == "--allow" && w[1] == "MCPTool(rea__*)")
        );
        // Gemini: a lista de permitidos é a do Treasury, mesmo com o template editado.
        let g = restrict_argv(
            Provider::GeminiCli,
            argv(&[
                "gemini",
                "--allowed-mcp-server-names",
                "github",
                "rea",
                "-p",
                "oi",
            ]),
            &policy,
        );
        let allowed: Vec<&String> = g
            .windows(2)
            .filter(|w| w[0] == "--allowed-mcp-server-names")
            .map(|w| &w[1])
            .collect();
        assert_eq!(allowed, vec!["treasury", "rea"], "{g:?}");
        assert!(!g.contains(&"github".to_string()));
        // Codex: só o não liberado é desligado.
        let c = restrict_argv(Provider::Codex, argv(&["codex", "exec", "oi"]), &policy);
        assert!(c.contains(&"mcp_servers.github.enabled=false".to_string()));
        assert!(!c.iter().any(|a| a.contains("mcp_servers.rea")));
        // Claude Code: o liberado entra no arquivo da conversa e nas ferramentas permitidas.
        let mut claude = policy.clone();
        claude
            .claude_user_mcps
            .insert("rea".into(), json!({"command": "npx", "args": ["rea"]}));
        let p =
            write_mcp_config(Provider::ClaudeCode, &t, dir.path(), dir.path(), &claude).unwrap();
        let cfg: Value = serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
        assert_eq!(cfg["mcpServers"]["rea"]["command"], "npx");
        let out = restrict_argv(Provider::ClaudeCode, argv(&["claude", "-p", "x"]), &claude);
        assert_eq!(
            value_after(&out, "--allowedTools"),
            vec!["mcp__treasury__*", "mcp__rea__*"]
        );
        // Sem nada liberado (o padrão), o Claude Code fica só com o Treasury.
        let p = write_mcp_config(
            Provider::ClaudeCode,
            &t,
            dir.path(),
            dir.path(),
            &CliPolicy::default(),
        )
        .unwrap();
        let cfg: Value = serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
        assert_eq!(cfg["mcpServers"].as_object().unwrap().len(), 1);
    }

    #[test]
    fn opencode_gets_only_treasury_mcp_in_its_own_server() {
        let dir = tempfile::tempdir().unwrap();
        let t = McpTarget {
            command: Path::new("/Applications/Treasury.app/Contents/MacOS/treasury"),
            home: None,
            token_file: Path::new("/tmp/x/token"),
        };
        let off = CliPolicy {
            user_mcps: vec!["github".into(), "treasury".into()],
            ..Default::default()
        };
        // 2.x: tudo negado menos o `execute` (Code Mode) e as ferramentas do Treasury.
        let c = write_mcp_config(Provider::OpencodeCli, &t, dir.path(), dir.path(), &off).unwrap();
        let cfg: Value = serde_json::from_str(&c).unwrap();
        let perm = &cfg["permission"];
        assert_eq!(perm["*"], "deny");
        assert_eq!(perm["execute"], "allow");
        assert_eq!(perm["treasury*"], "allow");
        assert!(perm.get("read").is_none());
        assert_eq!(cfg["agent"]["build"]["permission"], *perm);
        let srv = &cfg["mcp"]["servers"]["treasury"];
        assert_eq!(srv["command"][1], "mcp");
        assert!(srv["environment"].get(crate::mcp::TOKEN_FILE_ENV).is_some());
        assert_eq!(cfg["mcp"]["servers"]["github"]["disabled"], true);
        assert_eq!(cfg["share"], "disabled");
        // Nada no disco: a configuração vai pela variável de ambiente.
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
        let env = config_env(Provider::OpencodeCli, &c);
        assert_eq!(env[0].0, "OPENCODE_CONFIG_CONTENT");
        // Com o computador liberado, só a leitura volta.
        let on = CliPolicy {
            computer_access: true,
            ..Default::default()
        };
        let cfg: Value = serde_json::from_str(
            &write_mcp_config(Provider::OpencodeCli, &t, dir.path(), dir.path(), &on).unwrap(),
        )
        .unwrap();
        assert_eq!(cfg["permission"]["read"], "allow");
        assert!(cfg["permission"].get("shell").is_none());
        // 1.x: formato antigo de MCP.
        let v1 = CliPolicy {
            opencode_v1: true,
            ..off.clone()
        };
        let cfg: Value = serde_json::from_str(
            &write_mcp_config(Provider::OpencodeCli, &t, dir.path(), dir.path(), &v1).unwrap(),
        )
        .unwrap();
        assert_eq!(cfg["mcp"]["treasury"]["enabled"], true);
        assert_eq!(cfg["mcp"]["github"]["enabled"], false);

        let template = super::super::engine::OPENCODE_CLI_TEMPLATE;
        let mut vars = HashMap::new();
        vars.insert("prompt", "oi".to_string());
        vars.insert("model", "deepseek/deepseek-flash".to_string());
        let mut argv0 = expand(template, &vars).unwrap();
        argv0.extend(argv(&[
            "--auto",
            "--agent",
            "livre",
            "--server=http://x",
            "-f",
            "/etc/hosts",
        ]));
        let out = restrict_argv(Provider::OpencodeCli, argv0.clone(), &off);
        assert_eq!(
            out,
            argv(&[
                "opencode",
                "run",
                "--standalone",
                "--format",
                "json",
                "--model",
                "deepseek/deepseek-flash",
                "oi"
            ])
        );
        let out = restrict_argv(Provider::OpencodeCli, argv0, &v1);
        assert!(!out.contains(&"--standalone".to_string()));
        assert_eq!(value_after(&out, "--format"), vec!["json"]);
        assert_eq!(opencode_major("opencode v2.0.32"), Some(2));
        assert_eq!(opencode_major("1.14.3\n"), Some(1));
    }

    #[test]
    fn opencode_prompt_goes_by_stdin() {
        let dir = tempfile::tempdir().unwrap();
        let prompt = "Saldo da conta 1234: R$ 9.999,00";
        let none: &[super::super::agent::CliImage] = &[];
        let mut a = argv(&["opencode", "run", "--standalone", prompt]);
        let stdin = send_prompt(Provider::OpencodeCli, &mut a, prompt, none, dir.path())
            .unwrap()
            .unwrap();
        assert_eq!(a, argv(&["opencode", "run", "--standalone"]));
        assert_eq!(stdin, prompt.as_bytes());
    }

    #[test]
    fn parses_opencode_json_events() {
        let text = r#"{"type":"text","part":{"type":"text","text":"Saldo: R$ 10"}}"#;
        assert_eq!(
            parse_opencode_json(text),
            vec![CliEvent::Text("Saldo: R$ 10".into())]
        );
        // `execute` (Code Mode) e ferramentas do Treasury: o MCP já informa.
        let exec = r#"{"type":"tool_use","part":{"tool":"execute","callID":"c1","state":{"status":"completed","input":{"code":"return 1"}}}}"#;
        assert!(parse_opencode_json(exec).is_empty());
        let read = r#"{"type":"tool_use","part":{"tool":"read","callID":"c2","state":{"status":"error","input":{"filePath":"/x"},"error":"negado"}}}"#;
        let ev = parse_opencode_json(read);
        assert!(matches!(&ev[0], CliEvent::ToolUse { name, .. } if name == "read"));
        assert!(
            matches!(&ev[1], CliEvent::ToolResult { is_error: true, summary, .. } if summary == "negado")
        );
        let step = r#"{"type":"step_finish","part":{"tokens":{"input":196,"output":52,"reasoning":78,"cache":{"read":7680,"write":0}}}}"#;
        assert_eq!(
            parse_opencode_json(step),
            vec![CliEvent::Usage(Tokens {
                input: 7876,
                output: 130,
                cached: 7680,
                cache_write: 0
            })]
        );
        let err = r#"{"type":"error","error":{"type":"provider.no-route","message":"Model unavailable: x/y"}}"#;
        assert_eq!(
            parse_opencode_json(err),
            vec![CliEvent::Final {
                text: "Model unavailable: x/y".into(),
                is_error: true
            }]
        );
        assert!(parse_opencode_json("não é json").is_empty());
    }

    #[test]
    fn reads_opencode_mcp_names_from_jsonc() {
        let v1 = r#"{
            // comentário com "aspas"
            "$schema": "https://opencode.ai/config.json",
            "mcp": { "rea": {"type": "local", "command": ["npx"]}, /* outro */ "github": {"type": "remote", "url": "https://x//y"}, },
        }"#;
        assert_eq!(opencode_mcp_names(v1), vec!["github", "rea"]);
        let v2 = r#"{"mcp": {"servers": {"docs": {"type": "local", "command": ["x"]}}}}"#;
        assert_eq!(opencode_mcp_names(v2), vec!["docs"]);
        assert!(opencode_mcp_names("{").is_empty());
    }

    #[test]
    fn codex_disables_user_mcps_and_claude_skips_user_settings() {
        let pol = CliPolicy {
            user_mcps: vec!["github".into(), "treasury".into(), "nome estranho".into()],
            ..Default::default()
        };
        let out = restrict_argv(
            Provider::Codex,
            argv(&["codex", "exec", "--skip-git-repo-check", "oi"]),
            &pol,
        );
        assert_eq!(
            &out[..3],
            &["codex", "-c", "mcp_servers.github.enabled=false"]
        );
        assert!(
            !out.iter()
                .any(|a| a.contains("treasury.enabled") || a.contains("estranho"))
        );
        assert_eq!(value_after(&out, "--sandbox"), vec!["read-only"]);

        let std = expand(super::super::engine::CLAUDE_CODE_TEMPLATE, &vars("")).unwrap();
        let mut edited = std.clone();
        edited.push("--setting-sources".into());
        edited.push("user,project".into());
        let out = restrict_argv(Provider::ClaudeCode, edited, &CliPolicy::default());
        assert_eq!(value_after(&out, "--setting-sources"), vec!["project"]);
        assert!(out.contains(&"--strict-mcp-config".to_string()));
    }

    #[test]
    fn parses_mcp_servers_from_grok_inspect() {
        let text = "  Plugins (1)\n  └ chrome (user)\n\n  MCP Servers (3)\n  └ treasury (stdio)   config\n  └ chrome-devtools (stdio)  plugin: x\n  └ hearth-prd (stdio) ~/.claude.json [claude] [disabled]\n\n  LSP Servers (None)\n";
        assert_eq!(
            parse_inspect_mcps(text),
            vec!["treasury", "chrome-devtools", "hearth-prd"]
        );
    }

    #[test]
    fn user_mcps_reads_server_names_only() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".grok")).unwrap();
        std::fs::write(
            dir.path().join(".grok/config.toml"),
            "[mcp_servers.github]\ncommand = \"x\"\n[mcp_servers.\"linear\"]\nurl = \"https://l\"\n",
        )
        .unwrap();
        let mut names = grok_config_mcps(Some(dir.path()));
        names.sort();
        assert_eq!(names, vec!["github", "linear"]);
        assert!(grok_config_mcps(Some(&dir.path().join("nada"))).is_empty());
    }
}
