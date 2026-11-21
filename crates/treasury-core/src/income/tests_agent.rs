//! The script in which an agent (Grok through the CLI) could not register an attached payslip:
//! each error from that conversation, now with the output that leads the model to the right path.

use std::sync::Arc;

use serde_json::{Value, json};

use super::tests::HOLERITE;
use crate::api::{Actor, Api};
use crate::mcp::inproc::{AGENT_ACCESS, ToolOutcome, attach_turn_files, call_tool_in};
use crate::paths::TreasuryPaths;

fn setup() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Carlos"]}),
    )
    .unwrap();
    (dir, api)
}

/// How the app loop and the CLIs call: the attachment resolved by the conversation, then the tool.
fn tool(api: &Api, conv: &str, name: &str, args: Value) -> ToolOutcome {
    let actor = Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    match attach_turn_files(api, conv, name, args.clone()) {
        Ok(input) => call_tool_in(api, &actor, name, input, AGENT_ACCESS, Some(conv)),
        Err(m) => crate::mcp::inproc::rejected(name, &args, m),
    }
}

#[test]
fn the_payslip_conversation_now_leads_the_model_to_the_right_call() {
    let (_d, api) = setup();
    let conv = "conv-holerite";
    let att = Arc::new(
        crate::ai::attachments::prepare("2026-09.txt", None, HOLERITE.as_bytes().to_vec()).unwrap(),
    );
    api.ai_runtime().set_turn_files(conv, vec![att]);

    // The guide exists (also under the Portuguese name).
    for topic in ["payslip", "holerite"] {
        let o = tool(&api, conv, "get_playbook", json!({"topic": topic}));
        assert!(!o.is_error, "{}", o.content);
        assert!(o.content.contains("attachments"), "{}", o.content);
    }

    // Nothing: the conversation's only attachment.
    let o = tool(&api, conv, "preview_payslip", json!({}));
    assert!(!o.is_error, "{}", o.content);
    assert!(o.content.contains("2026-09"), "{}", o.content);

    // The made-up id in `files`: the list of attachments that exist.
    let o = tool(
        &api,
        conv,
        "preview_payslip",
        json!({"files": ["67c328ae1c"]}),
    );
    assert!(o.is_error);
    assert!(
        o.content.contains("1) 2026-09.txt") && o.content.contains("\"attachments\": [\"1\"]"),
        "{}",
        o.content
    );
    // The name in `files` (object without content) and the order in `attachments` work.
    for args in [
        json!({"files": [{"filename": "2026-09.txt"}]}),
        json!({"attachments": ["1"]}),
    ] {
        let o = tool(&api, conv, "preview_payslip", args);
        assert!(!o.is_error, "{}", o.content);
    }
    // `read_attachment` with a missing id lists the attachments (with no saved history,
    // says that history is off).
    let o = tool(&api, conv, "read_attachment", json!({"id": "67c328ae1c"}));
    assert!(o.is_error, "{}", o.content);

    // Text summarized by the model: says what was missing.
    let o = tool(
        &api,
        conv,
        "preview_payslip",
        json!({"text": "Demonstrativo de pagamento de salário setembro/2026, Carlos, salário e líquido no PDF"}),
    );
    assert!(o.is_error);
    assert!(
        o.content.contains("não achei o total bruto") && o.content.contains("Líquido a receber"),
        "{}",
        o.content
    );

    // Wrong parameter: the format with the required ones and an example.
    let o = tool(&api, conv, "create_account", json!({}));
    assert!(o.is_error);
    assert!(
        o.content.contains("obrigatórios `name`") && o.content.contains("Exemplo mínimo"),
        "{}",
        o.content
    );
    let o = tool(
        &api,
        conv,
        "import_payslip",
        json!({"account_id": "x", "gross": 1}),
    );
    assert!(o.is_error);
    assert!(
        o.content.contains("unknown field `account_id`"),
        "{}",
        o.content
    );
    assert!(o.content.contains("opcionais"), "{}", o.content);

    // Income without a payslip has its own tool (it becomes a proposal).
    let member = api.call("household.list_members", json!({})).unwrap()[0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let o = tool(
        &api,
        conv,
        "create_income_source",
        json!({"member_id": member, "name": "Salário Ailos", "kind": "clt",
               "gross_monthly": 446_086, "net_monthly": 204_334, "pay_day": 5}),
    );
    assert!(!o.is_error, "{}", o.content);
    assert!(o.note.proposal_id.is_some(), "{:?}", o.note);
}
