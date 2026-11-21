//! Ferramentas do MCP em processo, para o agente interno (motores de API e modelos
//! locais): os mesmos esquemas, validação, despacho, resumos e auditoria do servidor MCP,
//! sem transporte. Ver `ai::agent`.

use std::time::Instant;

use serde_json::Value;

use super::tools::{self, Access, ToolSpec};
use super::{Failure, audit, invalid_params};
use crate::ai::providers::ToolDef;
use crate::api::{Actor, Api};
use crate::i18n::{args, t, t_args};

/// Limite do texto devolvido ao modelo por ferramenta (contexto e custo contidos).
pub const MAX_TOOL_TEXT: usize = 24_000;

/// O que a interface mostra de uma chamada de ferramenta.
#[derive(Debug, Clone)]
pub struct ToolNote {
    pub name: String,
    pub command: String,
    /// Parâmetros resumidos (textos e listas cortados), para mostrar na conversa.
    pub args: String,
    pub summary: String,
    pub ok: bool,
    /// Proposta criada (escrita pendente de aprovação).
    pub proposal_id: Option<String>,
}

impl ToolNote {
    /// `base`: a moeda base do cofre, para os resumos com valores sem moeda própria.
    pub(crate) fn from_result(
        spec: &ToolSpec,
        data: &Value,
        params: &Value,
        base: Option<&str>,
    ) -> Self {
        Self {
            name: spec.name.into(),
            command: spec.command.into(),
            args: args_text(params),
            summary: spec.summarize_in(data, base),
            ok: true,
            proposal_id: (data.get("status").and_then(Value::as_str) == Some("pending_approval"))
                .then(|| {
                    data.get("proposal_id")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
                .flatten(),
        }
    }

    pub(crate) fn failed(spec: &ToolSpec, message: &str, params: &Value) -> Self {
        Self {
            name: spec.name.into(),
            command: spec.command.into(),
            args: args_text(params),
            summary: message.chars().take(300).collect(),
            ok: false,
            proposal_id: None,
        }
    }
}

fn args_text(params: &Value) -> String {
    let s = serde_json::to_string(params).unwrap_or_default();
    if s.chars().count() > 200 {
        format!("{}…", s.chars().take(199).collect::<String>())
    } else {
        s
    }
}

/// Resultado para o modelo e para a interface.
#[derive(Debug, Clone)]
pub struct ToolOutcome {
    pub note: ToolNote,
    /// Texto devolvido ao modelo (resumo + JSON, cortado).
    pub content: String,
    pub is_error: bool,
}

/// Ferramentas do modo enxuto (motor de plano gratuito com limite de tokens por minuto, ex.:
/// Groq): as consultas do dia a dia e as escritas mais pedidas, num pacote de ~4 mil tokens
/// em vez dos ~55 mil de todas.
pub const LEAN_TOOLS: &[&str] = &[
    "get_dashboard",
    "list_accounts",
    "query_transactions",
    "spending_summary",
    "get_budget_status",
    "get_monthly_flow",
    "list_categories",
    "get_insights",
    "get_daily_allowance",
    "list_goals",
    "get_income_summary",
    "categorize_transactions",
    "create_transaction",
    "remember",
    // Dados dos ativos com fonte (docs/agent-enrichment.md §10.3).
    "get_asset_data_gaps",
    "propose_asset_enrichment",
];

/// Ferramentas para o modelo, no formato neutro dos adaptadores.
pub fn tool_defs(access: Access) -> Vec<ToolDef> {
    tools::visible(access)
        .map(|t| ToolDef {
            name: t.name.into(),
            description: t
                .description
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" "),
            schema: t.input_schema(),
        })
        .collect()
}

/// Acesso do agente do app: tudo (escritas viram propostas; a memória de procedimentos é
/// dele e grava direto). O feedback ao desenvolvedor depende do modo desenvolvedor: veja
/// [`agent_access`].
pub const AGENT_ACCESS: Access = Access {
    writes: true,
    agent: true,
    dev: false,
    longevity: false,
    outside_brazil: false,
};

/// Modo desenvolvedor ligado (ajuste `developer_tools`) num cofre aberto para escrita.
pub fn developer_mode(api: &Api) -> bool {
    api.with_session(|s, _| {
        Ok(!s.db.is_read_only() && crate::devfeedback::developer_mode(s.db.conn())?)
    })
    .unwrap_or(false)
}

/// Linha da vida ligada no cofre aberto (as ferramentas dela só existem assim).
pub fn longevity_mode(api: &Api) -> bool {
    api.with_session(|s, _| Ok(crate::longevity::enabled(s.db.conn())))
        .unwrap_or(false)
}

/// A família do cofre aberto mora fora do Brasil (as ferramentas só do Brasil somem).
pub fn outside_brazil(api: &Api) -> bool {
    api.with_session(|s, _| Ok(!crate::country::is_brazil(s.db.conn())))
        .unwrap_or(false)
}

/// Acesso do agente do app agora: tudo, mais o feedback ao desenvolvedor no modo
/// desenvolvedor e a linha da vida com a função ligada, sem o que só vale no Brasil quando a
/// família mora fora.
pub fn agent_access(api: &Api) -> Access {
    Access {
        dev: developer_mode(api),
        longevity: longevity_mode(api),
        outside_brazil: outside_brazil(api),
        ..AGENT_ACCESS
    }
}

fn truncate(s: String, max: usize) -> String {
    if s.len() <= max {
        return s;
    }
    let mut cut = max;
    while !s.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}… [{}]", &s[..cut], t("mcp-resultado-cortado"))
}

/// Executa uma ferramenta fora de uma conversa (testes).
#[cfg(test)]
pub fn call_tool(api: &Api, actor: &Actor, name: &str, raw: Value, access: Access) -> ToolOutcome {
    call_tool_in(api, actor, name, raw, access, None)
}

/// Comando que recebe a conversa de origem (preenchida aqui, nunca pelo modelo).
pub(crate) const ORIGIN_COMMAND: &str = "ai.procedures.save";

/// Ferramentas que leem arquivos: `attachments` (nomes, ids ou a ordem dos anexos da
/// conversa) vira `files` com o conteúdo, sem o modelo repetir os bytes. Vale para o loop
/// do app e para as CLIs (pelo socket, com o job da conexão). Também aceita a referência
/// direto em `files` (`["1"]`, `["holerite.pdf"]`, `[{"filename": "holerite.pdf"}]` sem o
/// conteúdo), e, sem arquivo nem texto na chamada, a prévia usa o único anexo da conversa (a
/// importação, que vira proposta, pede o anexo nomeado). Uma
/// referência que não está na conversa é recusada com a lista dos anexos que estão.
pub(crate) fn attach_turn_files(
    api: &Api,
    key: &str,
    tool: &str,
    mut input: Value,
) -> Result<Value, String> {
    if !tools::reads_turn_files(tool) {
        return Ok(input);
    }
    let Some(obj) = input.as_object_mut() else {
        return Ok(input);
    };
    let mut refs: Vec<String> = match obj.remove("attachments") {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|v| match v {
                Value::String(s) => Some(s.clone()),
                Value::Number(n) => Some(n.to_string()),
                _ => None,
            })
            .collect(),
        Some(Value::String(s)) => vec![s],
        Some(Value::Number(n)) => vec![n.to_string()],
        _ => vec![],
    };
    let mut files = vec![];
    if let Some(Value::Array(given)) = obj.remove("files") {
        for f in given {
            match &f {
                Value::String(s) => refs.push(s.clone()),
                Value::Number(n) => refs.push(n.to_string()),
                // Objeto sem o conteúdo: referência pelo nome (ou id) do anexo.
                Value::Object(o) if o.get("content_base64").is_none_or(Value::is_null) => {
                    match ["id", "filename", "name"]
                        .iter()
                        .find_map(|k| o.get(*k).and_then(Value::as_str))
                    {
                        Some(r) => refs.push(r.to_string()),
                        None => files.push(f.clone()),
                    }
                }
                _ => files.push(f.clone()),
            }
        }
    }
    let rt = api.ai_runtime();
    let has_text = obj
        .get("text")
        .and_then(Value::as_str)
        .is_some_and(|t| !t.trim().is_empty());
    // Pagamentos já informados também contam: o anexo não entra por cima deles.
    let wants_texts = tools::reads_turn_texts(tool);
    let has_items = ["payments", "texts"].iter().any(|k| {
        obj.get(*k)
            .and_then(Value::as_array)
            .is_some_and(|a| !a.is_empty())
    });
    if refs.is_empty() && files.is_empty() && !has_text && !has_items {
        let have = rt.conversation_files(key);
        // Importar (proposta) só com o anexo nomeado: uma chamada vazia não grava nada.
        if tool.starts_with("import_") && obj.is_empty() && !have.is_empty() {
            return Err(t_args(
                "mcp-anexo-diga-qual",
                &args(&[("lista", attachment_list(&have).into())]),
            ));
        }
        // Nada na chamada: o único anexo da conversa (o caso comum do "leia este PDF").
        if let [(_, only)] = have.as_slice() {
            refs.push(only.id.clone());
        }
    }
    // Números além dos anexos do último turno ("1" a "14" com os prints mandados em duas
    // mensagens, até 10 por mensagem): a ordem é a da conversa toda.
    let nums: Vec<usize> = refs.iter().filter_map(|r| r.trim().parse().ok()).collect();
    if !nums.is_empty() && nums.len() == refs.len() {
        let have = rt.conversation_files(key);
        let in_turn = have.iter().filter(|(n, _)| n.is_some()).count();
        if nums.iter().any(|n| *n > in_turn) && nums.iter().all(|n| (1..=have.len()).contains(n)) {
            refs = nums.iter().map(|n| have[n - 1].1.id.clone()).collect();
        }
    }
    let mut missing = vec![];
    let mut texts = vec![];
    let mut sources = vec![];
    for r in &refs {
        match rt.turn_file(key, r) {
            // O texto do anexo (o OCR do print), não os bytes.
            Some(a) if wants_texts => match a.text.as_deref() {
                Some(t) if !t.trim().is_empty() => {
                    texts.push(Value::from(t.as_str()));
                    sources.push(Value::from(a.name.as_str()));
                }
                _ => {
                    return Err(t_args(
                        "mcp-anexo-sem-texto",
                        &args(&[("anexo", a.name.clone().into())]),
                    ));
                }
            },
            Some(a) => {
                use base64::Engine as _;
                // Arquivos de texto (OFX, CSV) chegam só com o texto.
                let bytes: &[u8] = if a.data.is_empty() {
                    a.text.as_deref().map_or(&[], |t| t.as_bytes())
                } else {
                    a.data.as_slice()
                };
                files.push(serde_json::json!({
                    "filename": a.name,
                    "content_base64": base64::engine::general_purpose::STANDARD.encode(bytes),
                }));
            }
            None => missing.push(r.clone()),
        }
    }
    if !missing.is_empty() {
        return Err(missing_attachments(&rt.conversation_files(key), &missing));
    }
    if !files.is_empty() {
        obj.insert("files".into(), Value::Array(files));
    }
    if !texts.is_empty() {
        let mut all = obj
            .remove("texts")
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();
        // Os nomes acompanham os textos (os informados antes ficam sem nome).
        let mut names: Vec<Value> = vec![Value::from(""); all.len()];
        names.extend(sources);
        all.extend(texts);
        obj.insert("texts".into(), Value::Array(all));
        obj.insert("sources".into(), Value::Array(names));
    }
    Ok(input)
}

/// Erro de anexo não encontrado, com os anexos que a conversa tem.
fn missing_attachments(
    have: &[(
        Option<usize>,
        std::sync::Arc<crate::ai::attachments::Attachment>,
    )],
    missing: &[String],
) -> String {
    let wanted = missing
        .iter()
        .map(|m| format!("\"{m}\""))
        .collect::<Vec<_>>()
        .join(", ");
    if have.is_empty() {
        return t_args("mcp-anexo-sem-anexos", &args(&[("anexos", wanted.into())]));
    }
    t_args(
        "mcp-anexo-nao-encontrado",
        &args(&[
            ("anexos", wanted.into()),
            ("lista", attachment_list(have).into()),
        ]),
    )
}

/// "1) extrato.pdf (id …); 2) …": os anexos da conversa, para a mensagem de erro.
fn attachment_list(
    have: &[(
        Option<usize>,
        std::sync::Arc<crate::ai::attachments::Attachment>,
    )],
) -> String {
    have.iter()
        .map(|(n, a)| match n {
            Some(n) => format!("{n}) {} (id {})", a.name, a.id),
            None => format!("{} (id {})", a.name, a.id),
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// Resultado de uma chamada recusada antes de chegar à ferramenta (ex.: anexo que não está
/// na conversa), no mesmo formato de um erro da ferramenta.
pub(crate) fn rejected(name: &str, raw: &Value, message: String) -> ToolOutcome {
    let params = audit::summarize_params(raw);
    let note = match tools::find(name) {
        Some(spec) => ToolNote::failed(spec, &message, &params),
        None => ToolNote {
            name: name.chars().take(64).collect(),
            command: String::new(),
            args: String::new(),
            summary: message.chars().take(300).collect(),
            ok: false,
            proposal_id: None,
        },
    };
    ToolOutcome {
        note,
        content: message,
        is_error: true,
    }
}

/// Comandos que leem a própria conversa do agente (histórico e anexos): a conversa vem do
/// job, nunca do modelo (não dá para ler outra conversa).
pub(crate) const CONVERSATION_COMMANDS: &[&str] = &["ai.chat_history", "ai.attachment_read"];

/// Preenche a conversa do job nos comandos que precisam dela e, na leitura de anexo, o
/// texto que está na memória da conversa (os anexos não ficam no cofre por padrão).
/// `find_tools` só devolve o que o cliente recebe: o servidor põe o acesso nos argumentos
/// (a entrada da ferramenta não tem esse campo, então o cliente não o escolhe).
pub(crate) fn inject_access(command: &str, args: &mut Value, access: Access) {
    if command == "ai.find_tools" {
        args["access"] =
            serde_json::to_value(super::find::AccessArgs::from(access)).unwrap_or_default();
    }
}

pub(crate) fn inject_conversation(
    api: &Api,
    command: &str,
    args: &mut Value,
    conversation_id: Option<&str>,
) {
    if command == ORIGIN_COMMAND {
        if let Some(c) = conversation_id {
            args["conversation_id"] = Value::from(c);
        }
        return;
    }
    if !CONVERSATION_COMMANDS.contains(&command) {
        return;
    }
    let Some(obj) = args.as_object_mut() else {
        return;
    };
    obj.remove("memory_text");
    obj.remove("memory_ids");
    match conversation_id {
        Some(c) => {
            obj.insert("conversation_id".into(), Value::from(c));
        }
        None => {
            obj.remove("conversation_id");
            return;
        }
    }
    let Some(c) = conversation_id else { return };
    if command == "ai.attachment_read" {
        let rt = api.ai_runtime();
        obj.insert(
            "memory_ids".into(),
            Value::from(rt.conversation_file_ids(c)),
        );
        // A posição em `read_attachment` é a da conversa toda (como na lista que ele mostra);
        // nome e id, pela memória. O texto da memória (o OCR do print) vale sobre o guardado.
        let find = |want: &str| match want.trim().parse::<usize>() {
            Ok(n) => rt
                .conversation_files(c)
                .get(n.checked_sub(1)?)
                .map(|(_, a)| a.clone()),
            Err(_) => rt.turn_file(c, want),
        };
        if let Some(want) = obj.get("id").and_then(Value::as_str).map(str::to_string)
            && let Some(a) = find(&want)
            && let Some(t) = &a.text
        {
            obj.insert("id".into(), Value::from(a.id.clone()));
            obj.insert("memory_text".into(), Value::from(t.as_str()));
        }
    }
}

/// Executa uma ferramenta (bloqueante; segura o banco só durante o comando). Dentro de uma
/// conversa, a memória de procedimentos guarda de onde veio cada nota.
pub fn call_tool_in(
    api: &Api,
    actor: &Actor,
    name: &str,
    raw: Value,
    access: Access,
    conversation_id: Option<&str>,
) -> ToolOutcome {
    let started = Instant::now();
    let Some(spec) = tools::find(name).filter(|s| access.allows_tool(s)) else {
        return ToolOutcome {
            note: ToolNote {
                name: name.chars().take(64).collect(),
                command: String::new(),
                args: String::new(),
                summary: t("mcp-ferramenta-desconhecida-curto"),
                ok: false,
                proposal_id: None,
            },
            content: t_args(
                "mcp-ferramenta-desconhecida-agente",
                &args(&[("ferramenta", name.to_string().into())]),
            ),
            is_error: true,
        };
    };
    let params = audit::summarize_params(&raw);
    let mut inbox_ids = vec![];
    let outcome: Result<Value, Failure> = super::inbox_files::resolve(api, spec.name, raw)
        .and_then(|(raw, ids)| {
            inbox_ids = ids;
            spec.prepare(raw)
        })
        .map_err(|m| Failure {
            code: "invalid".into(),
            message: invalid_params(spec.name, &m),
        })
        .and_then(|mut p| {
            inject_conversation(api, spec.command, &mut p.args, conversation_id);
            inject_access(spec.command, &mut p.args, access);
            let v = api
                .call_as(actor, spec.command, p.args)
                .map_err(|e| Failure::from_error(&e, spec.command))?;
            super::inbox_files::mark_imported(api, spec.name, &inbox_ids);
            Ok(spec.structured((p.post)(v)))
        });
    let client = match actor {
        Actor::Mcp { client, .. } => client.clone(),
        _ => "agente".into(),
    };
    audit::record(
        Some(api),
        None,
        audit::AuditRecord {
            client,
            tool: spec.name.into(),
            params: params.clone(),
            error: outcome
                .as_ref()
                .err()
                .map(|f| (f.code.clone(), f.message.clone())),
            duration_ms: started.elapsed().as_millis().min(i64::MAX as u128) as i64,
        },
    );
    match outcome {
        Ok(data) => {
            let base = tools::vault_base_currency(api);
            let note = ToolNote::from_result(spec, &data, &params, base.as_deref());
            let mut content = note.summary.clone();
            content.push('\n');
            content.push_str(&serde_json::to_string(&data).unwrap_or_default());
            ToolOutcome {
                note,
                content: truncate(content, MAX_TOOL_TEXT),
                is_error: false,
            }
        }
        Err(f) => ToolOutcome {
            note: ToolNote::failed(spec, &f.message, &params),
            content: f.message,
            is_error: true,
        },
    }
}
