//! Cadastro em lote pelo agente: lançamentos e investimentos lidos de uma página viram uma
//! proposta só, com o relatório do que foi criado e do que foi pulado; repetir não duplica.

use serde_json::{Value, json};

use super::*;
use crate::api::{Actor, Api};
use crate::paths::TreasuryPaths;

fn setup() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new_unlocked_process(paths).unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
    )
    .unwrap();
    (dir, api)
}

fn agent() -> Actor {
    Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    }
}

fn run(api: &Api, name: &str, args: Value) -> (Value, String) {
    let o = inproc::call_tool(api, &agent(), name, args, inproc::AGENT_ACCESS);
    assert!(!o.is_error, "{name}: {}", o.content);
    let (summary, data) = o.content.split_once('\n').unwrap();
    (serde_json::from_str(data).unwrap(), summary.to_string())
}

fn approve(api: &Api, p: &Value) {
    assert_eq!(p["status"], "pending_approval", "{p}");
    api.call("ai.approve", json!({"ids": [p["proposal_id"]]}))
        .unwrap();
}

fn count(api: &Api, sql: &str) -> i64 {
    api.with_session(|s, _| Ok(s.db.conn().query_row(sql, [], |r| r.get(0))?))
        .unwrap()
}

#[test]
fn investments_from_a_page_become_one_proposal_without_duplicates() {
    let (_d, api) = setup();
    let args = json!({
        "source": "XP, Minha carteira, 15/10/2026",
        "account": {"name": "XP Investimentos", "kind": "brokerage", "institution_name": "XP Investimentos"},
        "assets": [
            {"ticker": "PETR4", "name": "Petrobras PN", "class": "stock_br", "market": "B3"},
            {"ticker": "HGLG11", "name": "CSHG Logística", "class": "fii", "market": "B3"}
        ],
        "trades": [
            {"asset": "PETR4", "date": "2026-03-10", "side": "buy", "quantity": "100", "price": "30.50", "fees": 490},
            {"asset": "PETR4", "date": "2026-05-02", "side": "sell", "quantity": "40", "price": "35"},
            {"asset": "VALE3", "date": "2026-05-02", "side": "buy", "quantity": "1", "price": "60"}
        ],
        "positions": {"date": "2026-10-15", "items": [
            {"asset": "HGLG11", "quantity": "12", "average_price": "158.20"},
            {"asset": "PETR4", "quantity": "60", "average_price": "30.50"}
        ]},
        "income": [
            {"asset": "HGLG11", "kind": "fii_income", "pay_date": "2026-10-14", "gross": 1320}
        ]
    });
    let (p, summary) = run(&api, "import_investments", args.clone());
    // Uma proposta só, nada gravado ainda, e o relatório chega ao agente.
    assert!(summary.contains("Lote proposto"), "{summary}");
    assert_eq!(p["report"]["trades_created"], 2);
    assert_eq!(
        p["report"]["positions_adjusted"], 1,
        "PETR4 já confere com as operações"
    );
    assert_eq!(p["report"]["income_created"], 1);
    let skipped = p["report"]["skipped"].to_string();
    assert!(skipped.contains("VALE3 não cadastrado"), "{skipped}");
    assert!(skipped.contains("já confere"), "{skipped}");
    assert_eq!(count(&api, "SELECT count(*) FROM trade"), 0);
    assert_eq!(count(&api, "SELECT count(*) FROM ai_proposal"), 1);
    approve(&api, &p);
    assert_eq!(count(&api, "SELECT count(*) FROM asset"), 2);
    assert_eq!(count(&api, "SELECT count(*) FROM trade"), 3);
    assert_eq!(count(&api, "SELECT count(*) FROM income_event"), 1);
    let inst: String = api
        .with_session(|s, _| {
            Ok(s.db.conn().query_row(
                "SELECT i.name FROM account a JOIN institution i ON i.id = a.institution_id",
                [],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    assert_eq!(inst, "XP Investimentos");
    // A posição informada não mexe no caixa.
    assert_eq!(
        count(&api, "SELECT count(*) FROM trade WHERE txn_id IS NULL"),
        1
    );

    // De novo, com a conta pelo nome: nada novo.
    let (again, summary) = run(&api, "import_investments", args);
    assert_eq!(again["status"], "nothing_new", "{again}");
    assert!(summary.contains("Nada novo"), "{summary}");
    assert_eq!(
        again["report"]["assets_existing"].as_array().unwrap().len(),
        2
    );

    // Posição menor que a registrada: não vende sozinho.
    let acc: String = api
        .with_session(|s, _| {
            Ok(s.db
                .conn()
                .query_row("SELECT id FROM account", [], |r| r.get(0))?)
        })
        .unwrap();
    let (p, _) = run(
        &api,
        "import_investments",
        json!({"source": "x", "account_id": acc, "positions": {"date": "2026-10-16", "items": [
            {"asset": "HGLG11", "quantity": "5", "average_price": "150"}
        ]}}),
    );
    assert_eq!(p["status"], "nothing_new");
    assert!(
        p["report"]["skipped"]
            .to_string()
            .contains("registre a venda")
    );
}

#[test]
fn transactions_from_a_page_are_deduplicated_like_a_statement() {
    let (_d, api) = setup();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Itaú", "kind": "checking"}),
        )
        .unwrap()["id"]
        .clone();
    api.call(
        "transactions.create",
        json!({"account_id": acc, "date": "2026-09-03", "amount": -1250, "kind": "expense",
               "description": "PADARIA SAO JOAO"}),
    )
    .unwrap();
    let rows = json!([
        {"date": "2026-09-03", "amount": -1250, "description": "PADARIA SÃO JOÃO"},
        {"date": "2026-09-05", "amount": 500000, "description": "SALARIO"},
        {"date": "2026-09-06", "amount": -8990, "description": "MERCADO", "external_id": "abc1"},
        {"date": "06/09/2026", "amount": -100, "description": "DATA ERRADA"}
    ]);
    let o = inproc::call_tool(
        &api,
        &agent(),
        "import_transactions",
        json!({"account_id": acc, "source": "extrato Itaú set/2026", "rows": rows}),
        inproc::AGENT_ACCESS,
    );
    // A data fora do padrão é recusada antes de chegar ao núcleo.
    assert!(
        o.is_error && o.content.contains("rows[3].date"),
        "{}",
        o.content
    );
    let rows = json!([
        {"date": "2026-09-03", "amount": -1250, "description": "PADARIA SÃO JOÃO"},
        {"date": "2026-09-05", "amount": 500000, "description": "SALARIO"},
        {"date": "2026-09-06", "amount": -8990, "description": "MERCADO", "external_id": "abc1"}
    ]);
    let args = json!({"account_id": acc, "source": "extrato Itaú set/2026", "rows": rows});
    let (p, summary) = run(&api, "import_transactions", args.clone());
    assert!(
        summary.contains("2 novo(s)") && summary.contains("1 possível"),
        "{summary}"
    );
    assert_eq!(
        p["report"]["not_imported"][0]["status"],
        "possible_duplicate"
    );
    approve(&api, &p);
    assert_eq!(count(&api, "SELECT count(*) FROM txn"), 3);
    // Repetir: nada novo (mesmo identificador e mesma impressão digital).
    let (again, _) = run(&api, "import_transactions", args);
    assert_eq!(again["status"], "nothing_new", "{again}");
    assert_eq!(count(&api, "SELECT count(*) FROM import_batch"), 1);
}

#[test]
fn playbooks_are_listed_and_read() {
    let (_d, api) = setup();
    let (list, summary) = run(&api, "get_playbook", json!({}));
    assert!(list["playbooks"].as_array().unwrap().len() >= 8);
    assert!(summary.contains("guias"));
    let (p, _) = run(&api, "get_playbook", json!({"topic": "foreign_broker"}));
    assert!(p["body"].as_str().unwrap().contains("Avenue"));
    let o = inproc::call_tool(
        &api,
        &agent(),
        "get_playbook",
        json!({"topic": "nada"}),
        inproc::AGENT_ACCESS,
    );
    assert!(o.is_error && o.content.contains("investments"));
}

#[test]
fn delete_item_is_always_a_sensitive_proposal() {
    let (_d, api) = setup();
    let (p, _) = run(
        &api,
        "import_investments",
        json!({"source": "t", "account": {"name": "Nu", "kind": "brokerage", "institution_name": "Nu"},
               "assets": [{"ticker": "VALE3", "name": "Vale ON", "class": "stock_br", "market": "B3"}],
               "trades": [{"asset": "VALE3", "date": "2026-01-05", "side": "buy", "quantity": "10", "price": "60"}]}),
    );
    approve(&api, &p);
    // Modo autônomo ligado: apagar continua pedindo aprovação.
    api.call("ai.settings.set", json!({"autonomous": true}))
        .unwrap();
    let trade: String = api
        .with_session(|s, _| {
            Ok(s.db
                .conn()
                .query_row("SELECT id FROM trade", [], |r| r.get(0))?)
        })
        .unwrap();
    let (p, summary) = run(&api, "delete_item", json!({"kind": "trade", "id": trade}));
    assert!(summary.contains("Proposta"), "{summary}");
    let prop = api
        .call("ai.get_proposal", json!({"id": p["proposal_id"]}))
        .unwrap();
    assert_eq!(prop["level"], "sensitive");
    assert_eq!(prop["command"], "portfolio.delete_trade");
    assert_eq!(
        count(&api, "SELECT count(*) FROM trade"),
        1,
        "nada some antes da aprovação"
    );
    approve(&api, &p);
    assert_eq!(count(&api, "SELECT count(*) FROM trade"), 0);
    // Id que não existe: erro claro, nenhuma proposta.
    let o = inproc::call_tool(
        &api,
        &agent(),
        "delete_item",
        json!({"kind": "asset", "id": "nada"}),
        inproc::AGENT_ACCESS,
    );
    assert!(o.is_error, "{}", o.content);
    // Evento corporativo: registrar e listar.
    let asset: String = api
        .with_session(|s, _| {
            Ok(s.db
                .conn()
                .query_row("SELECT id FROM asset", [], |r| r.get(0))?)
        })
        .unwrap();
    let (p, _) = run(
        &api,
        "record_corporate_action",
        json!({"asset_id": asset, "date": "2026-03-01", "kind": "split", "factor": "2"}),
    );
    approve(&api, &p);
    let (list, summary) = run(&api, "list_corporate_actions", json!({"asset_id": asset}));
    assert!(summary.contains("1 evento"), "{summary}");
    assert_eq!(list["actions"][0]["kind"], "split");
}

#[test]
fn brokerage_notes_preview_and_import_through_the_tools() {
    use base64::Engine as _;
    let (_d, api) = setup();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Corretora", "kind": "brokerage"}),
        )
        .unwrap()["id"]
        .clone();
    api.call(
        "assets.create",
        json!({"ticker": "PETR4", "name": "Petrobras", "class": "stock_br"}),
    )
    .unwrap();
    let hglg = api
        .call(
            "assets.create",
            json!({"ticker": "HGLG11", "name": "CSHG Logística FII", "class": "fii"}),
        )
        .unwrap()["id"]
        .clone();
    api.call(
        "portfolio.create_trade",
        json!({"asset_id": hglg, "account_id": acc, "date": "2026-09-01", "side": "buy",
               "quantity": "10", "price": "150", "cash_movement": false}),
    )
    .unwrap();
    let note = crate::import::notes::tests::NOTE;
    let pdf = crate::import::notes::tests::pdf_with_lines(&note.lines().collect::<Vec<_>>());
    let files = json!([{"filename": "nota.pdf",
        "content_base64": base64::engine::general_purpose::STANDARD.encode(&pdf)}]);
    let (p, summary) = run(
        &api,
        "preview_brokerage_notes",
        json!({"account_id": acc, "files": files}),
    );
    assert!(summary.contains("1 nota(s) lida(s)"), "{summary}");
    assert_eq!(p["trades"].as_array().unwrap().len(), 3);
    // Sem arquivos: erro claro. Anexos só valem no agente do app (trocados antes).
    let o = inproc::call_tool(
        &api,
        &agent(),
        "preview_brokerage_notes",
        json!({"account_id": acc, "attachments": ["nota.pdf"]}),
        inproc::AGENT_ACCESS,
    );
    assert!(o.is_error && o.content.contains("files"), "{}", o.content);
    // Importar vira proposta sensível; aprovada, grava as operações.
    let (p, summary) = run(
        &api,
        "import_brokerage_notes",
        json!({"account_id": acc, "files": files}),
    );
    assert!(summary.contains("Proposta"), "{summary}");
    let prop = api
        .call("ai.get_proposal", json!({"id": p["proposal_id"]}))
        .unwrap();
    assert_eq!(prop["level"], "sensitive");
    approve(&api, &p);
    let imported = count(
        &api,
        "SELECT count(*) FROM trade WHERE note_number = '123456'",
    );
    assert!(imported >= 2, "{imported}");
    // De novo: nada novo (deduplica pela nota).
    let (p, _) = run(
        &api,
        "preview_brokerage_notes",
        json!({"account_id": acc, "files": files}),
    );
    assert!(
        p["trades"]
            .as_array()
            .unwrap()
            .iter()
            .all(|t| t["status"] != "new"),
        "{p}"
    );
}

#[test]
fn statement_file_import_through_the_tool() {
    use base64::Engine as _;
    let (_d, api) = setup();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Nubank", "kind": "checking"}),
        )
        .unwrap()["id"]
        .clone();
    let ofx = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/import/fixtures/nubank_xml.ofx"),
    )
    .unwrap();
    let files = json!([{"filename": "extrato.ofx",
        "content_base64": base64::engine::general_purpose::STANDARD.encode(&ofx)}]);
    // Sem a conta: o OFX não acha nenhuma com esse número e pede o account_id.
    let o = inproc::call_tool(
        &api,
        &agent(),
        "import_statement_file",
        json!({"files": files}),
        inproc::AGENT_ACCESS,
    );
    assert!(
        o.is_error && o.content.contains("informe account_id"),
        "{}",
        o.content
    );
    // Com agência e conta no cadastro, a conta é reconhecida sozinha.
    api.call(
        "accounts.update",
        json!({"id": acc, "branch": "0001", "account_number": "9876543-2"}),
    )
    .unwrap();
    let (p, summary) = run(&api, "import_statement_file", json!({"files": files}));
    assert!(
        summary.contains("Lote proposto") && summary.contains("novo(s)"),
        "{summary}"
    );
    approve(&api, &p);
    assert!(count(&api, "SELECT count(*) FROM txn") > 0);
    // O mesmo arquivo de novo: nada novo.
    let (again, _) = run(
        &api,
        "import_statement_file",
        json!({"account_id": acc, "files": files}),
    );
    assert_eq!(again["status"], "nothing_new", "{again}");
    // Dois arquivos de uma vez: recusado antes de chegar ao núcleo.
    let two = json!([files[0], files[0]]);
    let o = inproc::call_tool(
        &api,
        &agent(),
        "import_statement_file",
        json!({"account_id": acc, "files": two}),
        inproc::AGENT_ACCESS,
    );
    assert!(
        o.is_error && o.content.contains("exatamente um"),
        "{}",
        o.content
    );
}

#[test]
fn overdraft_through_the_account_tools() {
    let (_d, api) = setup();
    let (p, _) = run(
        &api,
        "create_account",
        json!({"name": "Cooperativa CC", "kind": "checking", "institution_name": "Cooperativa Exemplo",
               "opening_balance": -100000, "branch": "1234", "account_number": "12.345-6",
               "overdraft": {"limit": 500000, "monthly_rate": "8,00", "discount_pct": "25",
                             "iof_daily_rate": "0,0082", "iof_additional_rate": "0,38"}}),
    );
    approve(&api, &p);
    let acc = api.call("accounts.list", json!({})).unwrap()[0].clone();
    assert_eq!(acc["overdraft"]["limit"], 500000);
    assert_eq!(acc["branch"], "1234");
    assert_eq!(acc["account_number"], "12345-6");
    assert_eq!(acc["overdraft_status"]["used"], 100000);
    assert_eq!(acc["overdraft_status"]["estimated_interest"], 6000);
    // Remover o limite é alterar a conta: sensível.
    let (p, _) = run(
        &api,
        "update_account",
        json!({"id": acc["id"], "clear": ["overdraft"]}),
    );
    approve(&api, &p);
    assert!(api.call("accounts.list", json!({})).unwrap()[0]["overdraft"].is_null());
    // Em cartão, recusado.
    let o = inproc::call_tool(
        &api,
        &agent(),
        "create_account",
        json!({"name": "C", "kind": "savings", "overdraft": {"limit": 1}}),
        inproc::AGENT_ACCESS,
    );
    let p: Value = serde_json::from_str(o.content.split_once('\n').map_or("null", |x| x.1))
        .unwrap_or(Value::Null);
    if !o.is_error {
        // A proposta não chega a existir: o núcleo recusa na simulação.
        assert!(p.get("proposal_id").is_none(), "{p}");
    }
}

/// Relato de 19/10/2026: a escrita respondeu "pendente" e, segundos depois, o histórico já
/// mostrava o conjunto aplicado (o usuário tinha aprovado na fila). O agente agora vê de onde
/// veio o conjunto e o que aconteceu com a proposta.
#[test]
fn the_agent_can_see_that_its_proposal_was_approved() {
    let (_d, api) = setup();
    let (p, summary) = run(
        &api,
        "create_account",
        json!({"name": "Conta Ailos", "kind": "checking"}),
    );
    assert_eq!(p["status"], "pending_approval");
    assert!(summary.contains("list_pending_proposals"), "{summary}");
    // Ainda pendente: o histórico mostra o conjunto pendente com a proposta.
    let (h, _) = run(&api, "get_change_history", json!({}));
    let set = h["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["proposal_id"] == p["proposal_id"])
        .unwrap()
        .clone();
    assert_eq!(set["status"], "pending");

    // O usuário aprova no app.
    approve(&api, &p);
    let (pending, _) = run(&api, "list_pending_proposals", json!({}));
    assert!(pending["proposals"].as_array().unwrap().is_empty());
    let (all, _) = run(&api, "list_pending_proposals", json!({"status": "all"}));
    let mine = all["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["id"] == p["proposal_id"])
        .unwrap();
    assert_eq!(mine["status"], "approved");
    assert!(mine["reviewed_at"].is_string());
    let (h, _) = run(&api, "get_change_history", json!({}));
    let set = h["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["proposal_id"] == p["proposal_id"])
        .unwrap();
    assert_eq!(set["status"], "applied");
    assert!(set["applied_at"].is_string());
    // Conjunto feito pelo usuário não tem proposta.
    api.call(
        "accounts.create",
        json!({"name": "Carteira", "kind": "cash", "opening_date": "2026-01-01"}),
    )
    .unwrap();
    let (h, _) = run(&api, "get_change_history", json!({}));
    assert!(
        h["changes"][0].get("proposal_id").is_none(),
        "{}",
        h["changes"][0]
    );
}

/// Relato de 19/10/2026 (import_payslip respondeu pendente e o histórico mostrou aplicado):
/// a importação de holerite pelo agente é escrita sensível e fica de fato pendente, mesmo com
/// o modo autônomo ligado. Nada entra no cofre até o usuário aprovar no app; aprovado, o
/// histórico passa a `applied` com o `proposal_id` da proposta.
#[test]
fn import_payslip_stays_pending_until_the_user_approves() {
    let (_d, api) = setup();
    let ana: String = api
        .with_session(|s, _| {
            Ok(s.db
                .conn()
                .query_row("SELECT id FROM member LIMIT 1", [], |r| r.get(0))?)
        })
        .unwrap();
    api.with_session(|s, _| {
        crate::settings::set(s.db.conn(), crate::api::policy::AUTONOMOUS_SETTING, &true)
    })
    .unwrap();
    let text = "\
Empresa: COOPERATIVA EXEMPLO CNPJ: 12.345.678/0001-90
Demonstrativo de Pagamento de Salário
Competência: 06/2026
Código Descrição Referência Proventos Descontos
1 Salário 30,00 R$ 6.000,00
903 INSS 14,00 -R$ 649,60
904 IRRF 27,50 -R$ 382,88
Totais R$ 6.000,00 -R$ 1.032,48
Líquido R$ 4.967,52
";
    let (p, summary) = run(
        &api,
        "import_payslip",
        json!({"text": text, "member_id": ana}),
    );
    assert_eq!(p["status"], "pending_approval", "{p}");
    assert!(summary.contains("list_pending_proposals"), "{summary}");
    // Nada aplicado: nem o holerite nem a renda criada pelo CNPJ.
    assert_eq!(count(&api, "SELECT count(*) FROM payslip"), 0);
    assert_eq!(count(&api, "SELECT count(*) FROM income_source"), 0);
    let (pending, _) = run(&api, "list_pending_proposals", json!({}));
    assert_eq!(pending["proposals"].as_array().unwrap().len(), 1);
    let (h, _) = run(&api, "get_change_history", json!({}));
    let set = h["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["proposal_id"] == p["proposal_id"])
        .unwrap()
        .clone();
    assert_eq!(set["status"], "pending");

    approve(&api, &p);
    assert_eq!(count(&api, "SELECT count(*) FROM payslip"), 1);
    let (h, _) = run(&api, "get_change_history", json!({}));
    let set = h["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["proposal_id"] == p["proposal_id"])
        .unwrap()
        .clone();
    assert_eq!(set["status"], "applied");
}

/// Relato de 17/10/2026: uma compra importada sem caixa (txn_id nulo) só se corrigia apagando
/// e lançando de novo. `update_trade` com `cash_movement` liga o débito na conta (proposta
/// sensível, aprovada pelo usuário, com desfazer) e desliga de novo, sem mexer na operação.
#[test]
fn update_trade_links_and_unlinks_the_cash_movement() {
    let (_d, api) = setup();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Corretora", "kind": "brokerage", "opening_date": "2026-01-01"}),
        )
        .unwrap()["id"]
        .clone();
    let asset = api
        .call(
            "assets.create",
            json!({"ticker": "HGLG11", "name": "CSHG Logística FII", "class": "fii"}),
        )
        .unwrap()["id"]
        .clone();
    let trade = api
        .call(
            "portfolio.create_trade",
            json!({"asset_id": asset, "account_id": acc, "date": "2026-09-01", "side": "buy",
                   "quantity": "10", "price": "150", "fees": 500, "cash_movement": false}),
        )
        .unwrap();
    let id = trade["id"].as_str().unwrap().to_string();
    let txn_of = |api: &Api| -> Option<String> {
        api.with_session(|s, _| {
            Ok(s.db
                .conn()
                .query_row("SELECT txn_id FROM trade WHERE id = ?1", [&id], |r| {
                    r.get(0)
                })?)
        })
        .unwrap()
    };
    assert!(txn_of(&api).is_none());

    // Ligar o caixa: proposta sensível; nada muda até aprovar.
    let (p, _) = run(
        &api,
        "update_trade",
        json!({"id": id, "cash_movement": true}),
    );
    assert_eq!(p["status"], "pending_approval");
    assert!(txn_of(&api).is_none());
    approve(&api, &p);
    let txn = txn_of(&api).expect("lançamento ligado");
    let amount: i64 = api
        .with_session(|s, _| {
            Ok(s.db
                .conn()
                .query_row("SELECT amount FROM txn WHERE id = ?1", [&txn], |r| r.get(0))?)
        })
        .unwrap();
    assert_eq!(amount, -(150_000 + 500), "10 × 150 + corretagem");
    assert_eq!(
        count(&api, "SELECT count(*) FROM trade"),
        1,
        "a mesma operação, com o mesmo id"
    );

    // Desfazer pelo histórico volta ao estado sem caixa.
    let (h, _) = run(&api, "get_change_history", json!({}));
    let set = h["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["proposal_id"] == p["proposal_id"])
        .unwrap()["id"]
        .clone();
    api.call("changes.undo", json!({"id": set})).unwrap();
    assert!(txn_of(&api).is_none());
    assert_eq!(
        count(&api, "SELECT count(*) FROM txn WHERE kind <> 'opening'"),
        0
    );

    // Refeito e depois desligado: o lançamento sai e a operação fica.
    api.call("changes.redo", json!({"id": set})).unwrap();
    assert!(txn_of(&api).is_some());
    let (p, _) = run(
        &api,
        "update_trade",
        json!({"id": id, "cash_movement": false}),
    );
    approve(&api, &p);
    assert!(txn_of(&api).is_none());
    assert_eq!(count(&api, "SELECT count(*) FROM trade"), 1);
}

/// Posição informada menor que a registrada (feedback 01a1a354): na caixinha de renda fixa
/// (PU 1) a diferença vira resgate sem mexer no caixa; numa ação fica de fora, pedindo a venda.
#[test]
fn smaller_position_redeems_fixed_income_and_skips_stocks() {
    let (_d, api) = setup();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Banco Exemplo", "kind": "brokerage"}),
        )
        .unwrap()["id"]
        .clone();
    let rdb = api
        .call(
            "assets.create",
            json!({"name": "Caixinha RDB", "class": "fixed_income", "account_id": acc,
                   "fixed_income": {"kind": "rdb", "indexer": "cdi", "rate": "100",
                                    "issue_date": "2026-01-02", "liquidity": "daily",
                                    "ir_exempt": false}}),
        )
        .unwrap()["id"]
        .clone();
    let stock = api
        .call(
            "assets.create",
            json!({"ticker": "BBAS3", "name": "Banco do Brasil ON", "class": "stock_br"}),
        )
        .unwrap()["id"]
        .clone();
    for (asset, q, price) in [(&rdb, "1000", "1"), (&stock, "50", "25")] {
        api.call(
            "portfolio.create_trade",
            json!({"asset_id": asset, "account_id": acc, "date": "2026-02-02", "side": "buy",
                   "quantity": q, "price": price, "cash_movement": false}),
        )
        .unwrap();
    }
    let (p, _) = run(
        &api,
        "import_investments",
        json!({
            "source": "app do banco, 01/11/2026",
            "account_id": acc,
            "positions": {"date": "2026-11-01", "items": [
                {"asset": rdb, "quantity": "600", "average_price": "1"},
                {"asset": "BBAS3", "quantity": "40", "average_price": "25"}
            ]}
        }),
    );
    assert_eq!(p["report"]["positions_adjusted"], 1, "{p}");
    let skipped = p["report"]["skipped"].to_string();
    assert!(skipped.contains("record_trade"), "{skipped}");
    approve(&api, &p);
    let sells = count(
        &api,
        "SELECT count(*) FROM trade WHERE side = 'sell' AND quantity = '400' AND txn_id IS NULL",
    );
    assert_eq!(sells, 1);
}
