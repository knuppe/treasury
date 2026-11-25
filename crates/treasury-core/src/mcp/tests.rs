//! MCP server contract tests, talking to it as a client (`rmcp`) over a pair of in-memory
//! streams or through the local socket.

use std::sync::Arc;

use rmcp::model::{
    CallToolRequestParams, CallToolResult, ClientCapabilities, ClientConfig,
    GetPromptRequestParams, Implementation, ReadResourceRequestParams,
};
use rmcp::service::RunningService;
use rmcp::{RoleClient, ServiceExt};
use serde_json::{Map, Value, json};

use super::inputs::*;
use super::*;
use crate::api::Api;
use crate::paths::TreasuryPaths;

const PASSWORD: &str = "senha-mestra-forte";

/// Query tools (what an external client gets by default).
const EXPECTED_TOOLS: [(&str, &str); 164] = [
    ("get_net_worth", "reports.net_worth"),
    ("list_accounts", "accounts.list"),
    ("get_account_balance", "accounts.get"),
    ("query_transactions", "transactions.query"),
    ("spending_summary", "reports.spending"),
    ("get_budget_status", "budget.get"),
    ("suggest_budget", "budget.suggest"),
    ("list_recurring", "recurrences.list"),
    ("list_card_statements", "cards.list_statements"),
    ("get_card_overview", "cards.overview"),
    ("get_card_rewards", "cards.rewards"),
    ("get_card_value", "cards.value"),
    ("list_loyalty_programs", "cards.loyalty_list"),
    ("list_card_plastics", "cards.list_plastics"),
    ("list_goals", "goals.list"),
    ("get_portfolio_by_goal", "goals.objectives"),
    ("list_investment_plans", "portfolio.plans"),
    ("list_pending_contributions", "portfolio.plans_pending"),
    ("get_education_plan", "goals.education_get"),
    ("simulate_education_plan", "goals.education_simulate"),
    ("get_education_inflation", "goals.education_inflation"),
    ("estimate_taxes_paid", "tax_burden.estimate"),
    ("get_city_tax_revenue", "tax_burden.city"),
    ("list_projects", "projects.list"),
    ("list_project_entries", "projects.entries"),
    ("get_change_history", "changes.list"),
    ("explain_value", "reports.explain"),
    ("get_monthly_flow", "reports.month_flow"),
    ("get_spend_groups", "reports.spend_groups"),
    ("list_categories", "categories.list"),
    ("list_members", "household.list_members"),
    ("get_dashboard", "reports.dashboard"),
    ("get_portfolio", "portfolio.summary"),
    ("get_concentration", "portfolio.concentration"),
    ("get_position", "portfolio.position"),
    ("get_returns", "portfolio.performance"),
    ("get_performance_bridge", "portfolio.bridge"),
    ("list_assets", "assets.list"),
    ("list_trades", "portfolio.list_trades"),
    ("get_trade_summary", "portfolio.trade_summary"),
    ("list_income_events", "portfolio.list_income"),
    ("dividend_calendar", "portfolio.dividend_calendar"),
    ("list_pending_income", "portfolio.pending_income"),
    ("get_income_ledger", "portfolio.income_ledger"),
    ("simulate_retirement", "goals.retirement"),
    ("estimate_inss", "goals.inss_estimate"),
    ("list_retirement_scenarios", "goals.retirement_scenarios"),
    ("project_fee_costs", "portfolio.fee_cost"),
    ("compare_fixed_income", "portfolio.compare_fixed_income"),
    (
        "simulate_fixed_income_sale",
        "portfolio.simulate_fixed_income_sale",
    ),
    ("get_portfolio_risk", "portfolio.risk"),
    ("rebalance_suggestion", "portfolio.rebalance"),
    ("get_passive_income", "portfolio.passive_income"),
    ("get_index_series", "indices.get_series"),
    ("get_inflation", "indices.factor"),
    ("get_country_catalog", "indices.catalog"),
    ("convert_currency", "fx.convert"),
    ("tax_month_summary", "tax.month"),
    ("annual_tax_report", "tax.year"),
    ("list_loans", "debt.list"),
    ("get_amortization_schedule", "debt.schedule"),
    ("simulate_loan_prepayment", "debt.simulate_prepayment"),
    ("get_iptu", "physical.iptu_list"),
    ("list_physical_assets", "physical.list"),
    ("get_physical_asset", "physical.get"),
    ("list_pending_proposals", "ai.list_proposals"),
    ("list_alerts", "ai.alerts.list"),
    ("get_fundamentals", "research.indicators"),
    ("run_screen", "research.screen"),
    ("rank_assets", "research.rank"),
    ("compare_assets", "research.compare"),
    ("fair_price", "research.fair_price"),
    ("list_corporate_documents", "research.documents"),
    ("read_corporate_document", "research.document_read"),
    ("get_asset_news", "research.news"),
    ("get_documents_status", "research.documents_status"),
    ("list_institutions", "household.list_institutions"),
    ("get_suggestions", "ai.suggestions"),
    ("recall_procedures", "ai.procedures.list"),
    ("get_playbook", "ai.playbook"),
    ("list_inbox", "import.inbox_list"),
    ("read_document", "import.read_document"),
    ("find_tools", "ai.find_tools"),
    ("list_corporate_actions", "portfolio.list_actions"),
    ("get_capital_shares", "accounts.capital_share"),
    ("get_fx_reference_rate", "fx.reference_rate"),
    ("list_balance_syncs", "accounts.list_syncs"),
    ("get_balance_sync_hints", "accounts.sync_hints"),
    (
        "list_transfer_candidates",
        "transactions.transfer_candidates",
    ),
    (
        "list_cross_account_duplicates",
        "transactions.cross_account_duplicates",
    ),
    (
        "list_operation_candidates",
        "transactions.operation_candidates",
    ),
    ("get_fundamentus", "research.fundamentus"),
    ("check_dividends", "research.dividends_check"),
    ("get_fii_result", "research.fii_result"),
    ("get_come_cotas", "portfolio.come_cotas"),
    ("get_asset_statement", "portfolio.statement"),
    ("preview_brokerage_notes", "notes.preview"),
    ("get_asset_score", "score.get"),
    ("get_asset_data_gaps", "assets.data_gaps"),
    ("get_asset_field_sources", "assets.field_sources"),
    ("lookup_asset_reference", "assets.enrich_lookup"),
    ("list_asset_studies", "assets.study_list"),
    ("get_asset_study", "assets.study_get"),
    ("get_price_history", "market.prices"),
    ("get_portfolio_scores", "score.portfolio"),
    ("get_financial_health", "health.score"),
    ("preview_pension_statement", "pension.statement_preview"),
    ("list_fgts_accounts", "fgts.list"),
    ("list_fgts_entries", "fgts.entries"),
    ("preview_fgts_statement", "fgts.statement_preview"),
    ("check_fgts", "fgts.check"),
    ("compare_fgts_yield", "fgts.yield_compare"),
    ("project_fgts", "fgts.project"),
    ("simulate_fgts_anniversary", "fgts.anniversary_simulate"),
    ("get_balance_forecast", "forecast.balance"),
    ("get_daily_allowance", "forecast.daily_allowance"),
    ("plan_debt_payoff", "debt.payoff_plan"),
    ("compare_loan_rates", "debt.market_rates"),
    ("get_insights", "reports.insights"),
    ("get_attention", "attention.list"),
    ("get_next_step", "attention.next_step"),
    ("get_month_close", "attention.month_close"),
    ("list_learned_rules", "rules.learned"),
    ("list_subscriptions", "subscriptions.list"),
    ("list_detected_installments", "installments.detected"),
    ("preview_card_statement_pdf", "import.card_pdf_preview"),
    ("get_income_summary", "income.summary"),
    ("simulate_termination", "income.simulate_termination"),
    ("preview_payslip", "income.payslip_preview"),
    ("preview_receipt", "receipts.preview"),
    ("get_purchases_summary", "receipts.summary"),
    ("get_basket_inflation", "food.basket_inflation"),
    ("simulate_scenario", "scenarios.simulate"),
    ("list_scenarios", "scenarios.list"),
    ("preview_pension_payments", "pension.payments_preview"),
    ("list_pension_payments", "pension.payments_list"),
    ("list_sales", "portfolio.sales"),
    ("list_screen_methods", "research.methods"),
    ("run_screen_method", "research.method_run"),
    ("explain_screen_match", "research.method_explain"),
    ("get_market_overview", "research.market_overview"),
    ("list_asset_tags", "research.tags"),
    ("list_asset_labels", "assets.labels_list"),
    ("get_annual_tax_return_draft", "tax.annual_draft"),
    ("list_darfs", "tax.list_darfs"),
    ("get_darf_late_charges", "tax.darf_charges"),
    ("simulate_carne_leao", "tax.carne_leao"),
    ("get_crypto_custody", "tax.crypto_custody"),
    ("get_physical_asset_gain", "tax.physical_gain"),
    ("list_income_reports", "tax_burden.income_reports"),
    ("get_retirement_accounts", "retirement.statuses"),
    ("get_debt_summary", "debt.summary"),
    ("simulate_loan_anticipation", "debt.simulate_anticipation"),
    ("simulate_loan_portability", "debt.simulate_portability"),
    ("get_consumption_summary", "food.summary"),
    ("list_products", "food.products"),
    ("get_product_price_history", "food.product_history"),
    ("list_purchases", "food.purchases"),
    ("get_balance_history", "accounts.balance_history"),
    ("list_payslips", "income.list_payslips"),
    ("list_rules", "rules.list"),
    ("list_fx_operations", "fx.operations"),
    ("get_card_future_commitments", "cards.future_commitments"),
    ("get_physical_asset_history", "physical.history"),
];

/// Writes (always proposals): the app's agent, or external clients with `mcp.allow_writes`.
const WRITE_TOOLS: [(&str, &str); 72] = [
    ("create_transaction", "transactions.create"),
    ("update_transaction", "transactions.update"),
    ("categorize_transactions", "transactions.set_category"),
    ("create_rule", "rules.create"),
    ("create_recurrence", "recurrences.create"),
    ("set_budget", "budget.set_lines"),
    ("create_goal", "goals.create"),
    ("record_trade", "portfolio.create_trade"),
    ("record_income_event", "portfolio.create_income"),
    ("record_income_by_ticker", "portfolio.quick_income"),
    ("remember", "ai.remember"),
    ("create_institution", "household.create_institution"),
    ("update_institution", "household.update_institution"),
    ("create_account", "accounts.create"),
    ("update_account", "accounts.update"),
    ("set_card_annual_fee", "cards.set_annual_fee"),
    ("set_card_reward_program", "cards.set_reward_program"),
    ("record_loyalty_entry", "cards.loyalty_add_entry"),
    ("create_category", "categories.create"),
    ("update_category", "categories.update"),
    ("create_asset", "assets.create"),
    ("update_asset", "assets.update"),
    ("propose_asset_enrichment", "assets.enrich_apply"),
    ("update_recurrence", "recurrences.update"),
    ("update_goal", "goals.update"),
    ("link_goal_source", "goals.link_source"),
    ("unlink_goal_source", "goals.unlink_source"),
    ("split_transaction", "transactions.update"),
    ("create_transfer", "transactions.create_transfer"),
    (
        "add_capital_share_balance",
        "accounts.capital_share_add_balance",
    ),
    ("create_exchange", "fx.exchange"),
    ("sync_balance", "accounts.sync_balance"),
    ("resolve_balance_sync", "accounts.resolve_sync"),
    ("link_transfers", "transactions.link_transfers"),
    ("unlink_transfer", "transactions.unlink_transfer"),
    ("link_operation", "transactions.link_operation"),
    ("unlink_operation", "transactions.unlink_operation"),
    ("record_iptu", "physical.iptu_save"),
    ("create_physical_asset", "physical.create"),
    ("update_physical_asset", "physical.update"),
    ("add_physical_valuation", "physical.add_valuation"),
    ("add_physical_improvement", "physical.add_improvement"),
    ("create_project", "projects.create"),
    ("add_project_item", "projects.add_item"),
    ("link_transactions_to_project", "projects.link"),
    ("import_transactions", "import.rows"),
    ("import_investments", "portfolio.bulk"),
    ("delete_item", "crud.delete"),
    ("update_trade", "portfolio.update_trade"),
    ("record_corporate_action", "portfolio.create_action"),
    ("import_brokerage_notes", "notes.commit"),
    ("import_statement_file", "import.file"),
    ("record_loan_payment", "debt.record_payment"),
    ("import_pension_statement", "pension.statement_import"),
    ("import_fgts_statement", "fgts.statement_import"),
    ("record_fgts_withdrawal", "fgts.record_withdrawal"),
    ("use_fgts_for_housing", "fgts.use_for_housing"),
    ("set_fgts_modality", "fgts.set_modality"),
    ("update_fgts_account", "fgts.update_account"),
    ("create_income_source", "income.create_source"),
    ("import_payslip", "income.payslip_import"),
    ("import_receipt", "receipts.import"),
    ("import_loan_statement", "debt.import_statement"),
    ("link_loan_payment", "debt.link_payment"),
    ("import_pension_payments", "pension.payments_import"),
    ("withdraw_proposal", "ai.withdraw_proposal"),
    ("save_screen", "research.screens.create"),
    ("watch_asset", "research.observe"),
    ("set_asset_labels", "assets.labels_set"),
    ("tag_as_concentration_group", "assets.label_update"),
    (
        "set_concentration_limit",
        "portfolio.set_concentration_limit",
    ),
    ("start_asset_study", "assets.study_request"),
];
/// Context and procedure memory of the agent (conversation history and attachments, notes): only
/// for the app's agent.
const MEMORY_TOOLS: [(&str, &str); 5] = [
    ("read_chat_history", "ai.chat_history"),
    ("read_attachment", "ai.attachment_read"),
    ("recall_memory", "ai.memory_recall"),
    ("remember_procedure", "ai.procedures.save"),
    ("record_procedure_outcome", "ai.procedures.outcome"),
];
/// Feedback to the developer: only with developer mode (`developer_tools`); writes directly.
const DEV_TOOLS: [(&str, &str); 11] = [
    ("list_dev_feedback", "devfeedback.list"),
    ("get_dev_feedback", "devfeedback.get"),
    ("add_dev_feedback", "devfeedback.add"),
    ("bump_dev_feedback", "devfeedback.bump"),
    ("resolve_dev_feedback", "devfeedback.resolve"),
    ("update_dev_feedback", "devfeedback.update"),
    ("set_dev_feedback_status", "devfeedback.set_status"),
    ("set_dev_feedback_priority", "devfeedback.set_priority"),
    ("link_dev_feedback", "devfeedback.link"),
    ("delete_dev_feedback", "devfeedback.delete"),
    ("export_dev_feedback", "devfeedback.markdown"),
];
/// From the feedback board, they only read.
const DEV_READS: [&str; 3] = [
    "list_dev_feedback",
    "get_dev_feedback",
    "export_dev_feedback",
];
/// Life line: only with the feature on in the vault (`longevity.enabled`); read-only.
const LONGEVITY_TOOLS: [(&str, &str); 2] = [
    ("get_member_outlook", "longevity.outlook"),
    ("get_family_outlook", "longevity.family"),
];

struct Vault {
    _dir: tempfile::TempDir,
    paths: TreasuryPaths,
    api: SharedApi,
}

fn vault() -> Vault {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Arc::new(Api::new_unlocked_process(paths.clone()).unwrap());
    api.call(
        "vault.create",
        json!({"password": PASSWORD, "household_name": "Família Teste", "members": ["Ana"]}),
    )
    .unwrap();
    Vault {
        _dir: dir,
        paths,
        api,
    }
}

/// Turns off the mandatory key (default since V139) to test only the socket token.
pub(super) fn allow_keyless(api: &Api) {
    let mut s = api.call("mcp.server_settings", json!({})).unwrap();
    s["require_api_key"] = json!(false);
    api.call("mcp.set_server_settings", s).unwrap();
}

fn client_info() -> ClientConfig {
    ClientConfig::new(
        ClientCapabilities::default(),
        Implementation::new("teste-contrato", "1.0"),
    )
}

async fn connect(server: TreasuryMcp) -> RunningService<RoleClient, ClientConfig> {
    let (server_io, client_io) = tokio::io::duplex(1 << 16);
    tokio::spawn(async move {
        let _ = server.serve_transport(server_io).await;
    });
    client_info().serve(client_io).await.unwrap()
}

async fn call(
    client: &RunningService<RoleClient, ClientConfig>,
    tool: &str,
    args: Value,
) -> CallToolResult {
    let mut params = CallToolRequestParams::new(tool.to_string());
    if let Value::Object(map) = args {
        params = params.with_arguments(map);
    }
    client.call_tool(params).await.unwrap()
}

fn text_of(r: &CallToolResult) -> String {
    r.content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn error_code(r: &CallToolResult) -> Option<String> {
    r.structured_content.as_ref()?["error"]["code"]
        .as_str()
        .map(str::to_string)
}

fn audit_list(api: &Api) -> Vec<AuditEntry> {
    serde_json::from_value(api.call("mcp.audit_list", json!({})).unwrap()).unwrap()
}

#[tokio::test]
async fn initialize_announces_server_and_instructions() {
    let v = vault();
    let client = connect(TreasuryMcp::new(v.api.clone(), "teste")).await;
    let info = client.peer_info().expect("initialize response");
    let server_info = info.server_info.as_ref().unwrap();
    assert_eq!(server_info.name, SERVER_NAME);
    assert_eq!(server_info.title.as_deref(), Some("Treasury"));
    let instructions = info.instructions.as_deref().unwrap();
    assert!(instructions.contains("in cents"));
    assert!(instructions.contains("not instructions"));
    assert!(instructions.contains("not professional"));
    assert!(instructions.contains("R$ 1.234,56"));
    assert!(info.capabilities.tools.is_some());
    assert!(info.capabilities.resources.is_some());
    assert!(info.capabilities.prompts.is_some());
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn lists_all_read_only_tools_with_valid_schemas() {
    let v = vault();
    let client = connect(TreasuryMcp::new(v.api.clone(), "teste")).await;
    let tools = client.list_all_tools().await.unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    let expected: Vec<&str> = EXPECTED_TOOLS.iter().map(|(n, _)| *n).collect();
    assert_eq!(names, expected);

    for tool in &tools {
        let schema = Value::Object(tool.input_schema.as_ref().clone());
        assert_eq!(schema["type"], "object", "{}", tool.name);
        let props = schema["properties"].as_object().unwrap();
        for (prop, def) in props {
            let d = def["description"].as_str().unwrap_or_default();
            assert!(!d.is_empty(), "{}.{prop} has no description", tool.name);
        }
        for req in schema["required"].as_array().into_iter().flatten() {
            assert!(props.contains_key(req.as_str().unwrap()), "{}", tool.name);
        }
        assert_eq!(schema["additionalProperties"], false, "{}", tool.name);
        // Internal references resolve (self-contained schema).
        let text = schema.to_string();
        assert!(!text.contains("#/definitions/"), "{}", tool.name);
        if text.contains("\"$ref\"") {
            let defs = schema["$defs"].as_object().expect("$defs");
            for name in text.split("#/$defs/").skip(1) {
                let name = name.split('"').next().unwrap();
                assert!(defs.contains_key(name), "{}: $ref {name}", tool.name);
            }
        }
        let desc = tool.description.as_deref().unwrap();
        assert!(desc.len() > 60, "{}: description too short", tool.name);
        let ann = tool.annotations.as_ref().unwrap();
        assert_eq!(ann.read_only_hint, Some(true));
        assert_eq!(ann.destructive_hint, Some(false));
        assert!(tool.title.is_some());
    }

    // Descriptions mention units and formats.
    let by_name = |n: &str| tools.iter().find(|t| t.name == n).unwrap().clone();
    assert!(
        by_name("get_net_worth")
            .description
            .unwrap()
            .contains("cents")
    );
    let q = by_name("query_transactions");
    assert!(q.description.as_deref().unwrap().contains("200"));
    let q_schema = Value::Object(q.input_schema.as_ref().clone());
    assert_eq!(q_schema["properties"]["limit"]["maximum"], 200);
    assert!(
        q_schema["properties"]["date_from"]["description"]
            .as_str()
            .unwrap()
            .contains("YYYY-MM-DD")
    );
    let s = Value::Object(by_name("spending_summary").input_schema.as_ref().clone());
    assert_eq!(s["required"], json!(["from", "to"]));
    let s = Value::Object(by_name("estimate_inss").input_schema.as_ref().clone());
    assert_eq!(s["properties"]["sex"]["enum"], json!(["male", "female"]));
    client.cancel().await.unwrap();
}

#[test]
fn each_tool_maps_to_a_read_only_command() {
    let v = vault();
    let reader = Actor::Mcp {
        client: "teste".into(),
        read_only: true,
    };
    for (spec, (name, cmd)) in tools::TOOLS.iter().zip(EXPECTED_TOOLS) {
        assert_eq!(spec.name, name);
        assert_eq!(spec.command, cmd, "{name}");
        // The dispatcher allows the command for read-only clients (it may not be implemented
        // yet, but it is never forbidden).
        let err = v.api.call_as(&reader, cmd, json!({})).err();
        assert!(
            !matches!(err, Some(crate::Error::Forbidden(_))),
            "{cmd} blocked for reading"
        );
    }
}

#[test]
fn inputs_are_coherent_with_core_dtos() {
    use crate::{changes, reports, transactions};

    let q = QueryTransactionsInput {
        date_from: Some("2026-01-01".into()),
        date_to: Some("2026-01-31".into()),
        basis: Some(Basis::Cash),
        account_ids: Some(vec!["a".into()]),
        category_ids: Some(vec!["c".into()]),
        include_subcategories: Some(false),
        member_ids: Some(vec!["m".into()]),
        kinds: Some(vec![
            TxnKind::Income,
            TxnKind::Expense,
            TxnKind::Transfer,
            TxnKind::CardPayment,
            TxnKind::Investment,
            TxnKind::Adjustment,
        ]),
        statuses: Some(vec![
            TxnStatus::Planned,
            TxnStatus::Cleared,
            TxnStatus::Reconciled,
        ]),
        text: Some("mercado".into()),
        amount_min: Some(100),
        amount_max: Some(5000),
        tag_ids: Some(vec!["t".into()]),
        project_ids: Some(vec!["p".into()]),
        import_id: Some("i".into()),
        statement_id: Some("s".into()),
        uncategorized: Some(true),
        sort: Some(TxnSort::AmountAsc),
        limit: Some(20),
        offset: Some(40),
    };
    let args = q.to_api_args().unwrap();
    let filter: transactions::TxnFilter = serde_json::from_value(args.clone()).unwrap();
    assert_eq!(filter.limit, Some(20));
    assert_eq!(filter.sort, Some(transactions::TxnSort::AmountAsc));
    assert_no_field_lost(&args, &serde_json::to_value(&filter).unwrap());
    for sort in [TxnSort::DateDesc, TxnSort::DateAsc, TxnSort::AmountDesc] {
        let _: transactions::TxnSort = serde_json::from_value(json!(sort)).unwrap();
    }

    for group_by in [
        GroupBy::Category,
        GroupBy::ParentCategory,
        GroupBy::Member,
        GroupBy::Account,
        GroupBy::Month,
    ] {
        let s = SpendingSummaryInput {
            from: "2026-01-01".into(),
            to: "2026-03-31".into(),
            basis: Some(Basis::Accrual),
            kind: Some(CategoryKind::Income),
            group_by: Some(group_by),
            member_ids: Some(vec!["m".into()]),
            account_ids: Some(vec!["a".into()]),
            category_ids: Some(vec!["c".into()]),
        };
        let args = s.to_api_args().unwrap();
        let dto: reports::SpendingArgs = serde_json::from_value(args.clone()).unwrap();
        assert_no_field_lost(&args, &serde_json::to_value(&dto).unwrap());
    }

    for metric in [
        Metric::AccountBalance,
        Metric::NetWorth,
        Metric::Spending,
        Metric::Income,
        Metric::SavingsRate,
    ] {
        let e = ExplainValueInput {
            metric,
            account_id: Some("a".into()),
            as_of: Some("2026-10-01".into()),
            from: Some("2026-09-01".into()),
            to: Some("2026-09-30".into()),
            basis: Some(Basis::Cash),
            category_id: Some("c".into()),
            member_id: Some("m".into()),
            month: Some("2026-09".into()),
            limit: Some(10),
        };
        let args = e.to_api_args().unwrap();
        let dto: reports::ExplainArgs = serde_json::from_value(args.clone()).unwrap();
        assert_eq!(serde_json::to_value(dto.metric).unwrap(), json!(metric));
        assert_eq!(dto.limit, Some(10));
    }

    for author in [
        AuthorKind::User,
        AuthorKind::Import,
        AuthorKind::Ai,
        AuthorKind::System,
    ] {
        let c = ChangeHistoryInput {
            limit: Some(5),
            offset: Some(1),
            author_kind: Some(author),
            table_name: Some("txn".into()),
            row_id: Some("x".into()),
        };
        let args = c.to_api_args().unwrap();
        let dto: changes::ChangeListArgs = serde_json::from_value(args.clone()).unwrap();
        assert_no_field_lost(&args, &serde_json::to_value(&dto).unwrap());
    }

    let b = BudgetStatusInput {
        month: None,
        basis: Some(Basis::Cash),
    }
    .to_api_args()
    .unwrap();
    assert!(b["month"].as_str().unwrap().len() == 7);
    let _: crate::budget::Basis = serde_json::from_value(b["basis"].clone()).unwrap();
}

#[test]
fn research_inputs_are_coherent_with_core_dtos() {
    use super::inputs_invest::AssetClass;
    use super::inputs_research::*;
    use crate::research::{api as rapi, screen, views};

    let f = GetFundamentalsInput {
        asset_id: None,
        ticker: Some("PETR4".into()),
    }
    .to_api_args()
    .unwrap();
    let dto: rapi::SubjectArgs = serde_json::from_value(f.clone()).unwrap();
    assert_no_field_lost(&f, &serde_json::to_value(&dto).unwrap());
    assert!(GetFundamentalsInput::default().to_api_args().is_err());

    for op in [
        CmpOp::Gt,
        CmpOp::Gte,
        CmpOp::Lt,
        CmpOp::Lte,
        CmpOp::Eq,
        CmpOp::Ne,
        CmpOp::Between,
        CmpOp::In,
        CmpOp::NotIn,
        CmpOp::Contains,
        CmpOp::Exists,
        CmpOp::Missing,
    ] {
        let _: screen::CmpOp = serde_json::from_value(json!(op)).unwrap();
    }
    let s = RunScreenInput {
        filters: Some(ScreenFilterInput {
            logic: Some(Logic::Or),
            conditions: Some(vec![ConditionInput {
                field: "pl".into(),
                op: CmpOp::Lt,
                value: Some(json!(10)),
            }]),
            tags: Some(vec!["nunca_prejuizo".into()]),
            groups: Some(vec![ConditionGroupInput {
                logic: None,
                conditions: vec![ConditionInput {
                    field: "dy".into(),
                    op: CmpOp::Between,
                    value: Some(json!([0.06, 0.2])),
                }],
            }]),
            universe: Some(UniverseFilterInput {
                classes: Some(vec![AssetClass::StockBr, AssetClass::Fii]),
                sectors: Some(vec!["energia".into()]),
                segments: None,
                min_liquidity: Some(1e6),
                only_mine: Some(false),
            }),
        }),
        screen_id: None,
        sort_by: Some("dy".into()),
        order: Some(SortOrder::Desc),
        limit: Some(1000),
        fields: Some(vec!["roe".into()]),
    }
    .to_api_args()
    .unwrap();
    let dto: screen::ScreenArgs = serde_json::from_value(s.clone()).unwrap();
    assert_eq!(dto.limit, Some(200));
    let q = dto.filters.clone().unwrap();
    assert_eq!(q.logic, screen::Logic::Or);
    assert_eq!(q.groups[0].logic, screen::Logic::And);
    assert_eq!(q.tags, vec!["nunca_prejuizo".to_string()]);
    screen::validate(&q).unwrap();
    assert_no_field_lost(&s, &serde_json::to_value(&dto).unwrap());
    assert_no_field_lost(&s["filters"], &serde_json::to_value(&q).unwrap());

    let r = RankAssetsInput {
        indicator: "pl".into(),
        order: Some(SortOrder::Asc),
        limit: Some(5),
        classes: Some(vec![AssetClass::StockBr]),
        sectors: Some(vec!["bancos".into()]),
        segments: Some(vec!["x".into()]),
        min_liquidity: Some(1.0),
        only_mine: Some(true),
    }
    .to_api_args()
    .unwrap();
    let dto: screen::RankArgs = serde_json::from_value(r.clone()).unwrap();
    assert_no_field_lost(&r, &serde_json::to_value(&dto).unwrap());
    assert_no_field_lost(
        &r["filters"],
        &serde_json::to_value(dto.filters.unwrap()).unwrap(),
    );

    let c = CompareAssetsInput {
        asset_ids: Some(vec!["a".into()]),
        tickers: Some(vec!["ITUB4".into()]),
    }
    .to_api_args()
    .unwrap();
    let _: views::CompareAssetsArgs = serde_json::from_value(c).unwrap();
    assert!(
        CompareAssetsInput {
            asset_ids: None,
            tickers: Some(vec!["A".into(); 7]),
        }
        .to_api_args()
        .is_err()
    );

    let fp = FairPriceInput {
        asset_id: None,
        ticker: Some("TAEE11".into()),
        bazin_rate: Some(0.08),
        years: Some(3),
    }
    .to_api_args()
    .unwrap();
    let dto: rapi::FairPriceArgs = serde_json::from_value(fp.clone()).unwrap();
    assert_no_field_lost(&fp, &serde_json::to_value(&dto).unwrap());
    assert!(
        FairPriceInput {
            ticker: Some("X".into()),
            bazin_rate: Some(6.0),
            ..Default::default()
        }
        .to_api_args()
        .is_err()
    );

    let d = CorporateDocumentsInput {
        asset_id: Some("a".into()),
        ticker: None,
        from: Some("2026-01-01".into()),
        to: Some("2026-10-01".into()),
        categories: Some(vec!["Fato Relevante".into()]),
        limit: Some(10),
    }
    .to_api_args()
    .unwrap();
    let _: views::DocumentsArgs = serde_json::from_value(d).unwrap();
    assert!(
        CorporateDocumentsInput {
            from: Some("ontem".into()),
            ..Default::default()
        }
        .to_api_args()
        .is_err()
    );
}

/// Every field sent by the tool exists in the core DTO (nothing is silently ignored).
fn assert_no_field_lost(sent: &Value, dto: &Value) {
    let dto = dto.as_object().unwrap();
    for key in sent.as_object().unwrap().keys() {
        assert!(dto.contains_key(key), "field {key} is not in the DTO");
    }
}

#[tokio::test]
async fn physical_asset_tools_read_the_core() {
    use super::inputs_invest::{ListPhysicalAssetsInput, PhysicalAssetIdInput};
    let v = vault();
    allow_keyless(&v.api);
    let a = v
        .api
        .call(
            "physical.create",
            json!({"name": "Casa da praia", "kind": "real_estate", "purchase_date": "2020-03-10",
                   "purchase_price": 45_000_000, "valuation_method": "none"}),
        )
        .unwrap();
    let id = a["id"].as_str().unwrap().to_string();
    // Inputs become the core's arguments without losing fields.
    let args = ListPhysicalAssetsInput {
        as_of: Some("2026-10-01".into()),
        deflator: Some(super::inputs_invest::Deflator::Ipca),
        include_archived: Some(true),
    }
    .to_api_args()
    .unwrap();
    let dto: crate::physical::PhysicalReadArgs = serde_json::from_value(args.clone()).unwrap();
    assert_no_field_lost(&args, &serde_json::to_value(&dto).unwrap());
    let g = PhysicalAssetIdInput {
        asset_id: format!(" {id} "),
        as_of: None,
        deflator: None,
    }
    .to_api_args()
    .unwrap();
    assert_eq!(g, json!({"id": id}));
    assert!(
        PhysicalAssetIdInput {
            asset_id: " ".into(),
            ..Default::default()
        }
        .to_api_args()
        .is_err()
    );

    let client = connect(TreasuryMcp::new(v.api.clone(), "teste")).await;
    let r = call(&client, "list_physical_assets", json!({})).await;
    assert_eq!(r.is_error, Some(false), "{}", text_of(&r));
    let data = r.structured_content.clone().unwrap();
    assert_eq!(data["count"], 1);
    assert_eq!(data["assets"][0]["figures"]["value"], 45_000_000);
    assert!(text_of(&r).contains("1 bem"));
    let r = call(&client, "get_physical_asset", json!({"asset_id": id})).await;
    assert_eq!(r.is_error, Some(false), "{}", text_of(&r));
    assert!(text_of(&r).contains("Casa da praia: R$ 450.000,00"));
    let r = call(
        &client,
        "get_physical_asset",
        json!({"asset_id": "nao-existe"}),
    )
    .await;
    assert_eq!(r.is_error, Some(true));
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn working_tool_returns_structured_data_and_is_audited() {
    let v = vault();
    allow_keyless(&v.api);
    let client = connect(TreasuryMcp::new(v.api.clone(), "teste")).await;
    let r = call(&client, "get_change_history", json!({"limit": 5})).await;
    assert_eq!(r.is_error, Some(false), "{}", text_of(&r));
    let data = r.structured_content.clone().unwrap();
    assert_eq!(data["count"], 1);
    assert_eq!(data["changes"][0]["reason"], "Criação do cofre");
    let text = text_of(&r);
    assert!(text.contains("1 conjunto de alterações"));
    assert!(text.contains("centavos"));
    // The full JSON also goes as text, for clients that ignore structuredContent.
    assert_eq!(r.content.len(), 2);
    let as_text: Value = serde_json::from_str(&r.content[1].as_text().unwrap().text).unwrap();
    assert_eq!(as_text, data);

    let entries = audit_list(&v.api);
    let e = entries
        .iter()
        .find(|e| e.tool == "get_change_history")
        .expect("audit entry");
    assert!(e.ok);
    assert_eq!(e.client, "teste-contrato via teste");
    assert_eq!(e.params, json!({"limit": 5}));
    assert!(e.error.is_none());
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn errors_are_readable_tool_errors() {
    let v = vault();
    allow_keyless(&v.api);
    let client = connect(TreasuryMcp::new(v.api.clone(), "teste")).await;

    // Local validation, before the API.
    let r = call(&client, "query_transactions", json!({"date_from": "ontem"})).await;
    assert_eq!(r.is_error, Some(true));
    assert_eq!(error_code(&r).as_deref(), Some("invalid"));
    assert!(text_of(&r).contains("AAAA-MM-DD"));
    let r = call(&client, "get_change_history", json!({"inexistente": 1})).await;
    assert_eq!(error_code(&r).as_deref(), Some("invalid"));
    let r = call(&client, "explain_value", json!({"metric": "spending"})).await;
    assert_eq!(error_code(&r).as_deref(), Some("invalid"));

    // Unknown tool is a protocol error.
    let err = client
        .call_tool(CallToolRequestParams::new("fx_set"))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("ferramenta desconhecida"));

    // All tools answer something readable (data or a tool error), never permission
    // denied.
    let samples = [
        ("get_net_worth", json!({})),
        ("list_accounts", json!({})),
        ("get_account_balance", json!({"account_id": "x"})),
        ("query_transactions", json!({})),
        (
            "spending_summary",
            json!({"from": "2026-01-01", "to": "2026-01-31"}),
        ),
        ("get_budget_status", json!({})),
        ("list_recurring", json!({})),
        ("list_card_statements", json!({"account_id": "x"})),
        ("list_card_plastics", json!({})),
        ("get_card_overview", json!({"account_id": "x"})),
        ("get_card_rewards", json!({"account_id": "x"})),
        ("list_loyalty_programs", json!({})),
        ("get_card_value", json!({})),
        ("list_goals", json!({})),
        ("get_education_plan", json!({"goal_id": "x"})),
        (
            "simulate_education_plan",
            json!({"birth_date": "2018-06-15", "fee": 250000}),
        ),
        ("get_education_inflation", json!({})),
        ("get_change_history", json!({})),
        ("explain_value", json!({"metric": "net_worth"})),
        ("get_monthly_flow", json!({})),
        (
            "get_spend_groups",
            json!({"from": "2026-01", "to": "2026-03"}),
        ),
        ("list_categories", json!({"kind": "expense"})),
        ("list_members", json!({})),
        ("get_dashboard", json!({"month": "2026-10"})),
        ("get_portfolio", json!({"deflator": "ipca"})),
        ("get_position", json!({"asset_id": "x"})),
        ("get_returns", json!({})),
        ("get_returns", json!({"method": "daily"})),
        ("get_performance_bridge", json!({})),
        ("list_assets", json!({})),
        ("list_trades", json!({})),
        ("get_trade_summary", json!({"asset_id": "x"})),
        ("list_income_events", json!({})),
        ("dividend_calendar", json!({})),
        ("list_pending_income", json!({})),
        ("get_income_ledger", json!({"only_problems": true})),
        ("simulate_retirement", json!({"current_age": 40})),
        (
            "estimate_inss",
            json!({"sex": "female", "contribution_years": 20.0, "average_salary": 500000,
                   "current_age": 50}),
        ),
        ("list_retirement_scenarios", json!({})),
        ("project_fee_costs", json!({})),
        (
            "compare_fixed_income",
            json!({"amount": 1000000, "offers": [
                {"kind": "cdb", "index": "cdi_pct", "rate": 110, "months": 24},
                {"kind": "lci", "index": "cdi_pct", "rate": 90, "months": 24}]}),
        ),
        (
            "simulate_fixed_income_sale",
            json!({"amount": 1000000, "start": "2026-01-05", "maturity": "2030-01-02",
                   "sale_date": "2028-01-05", "index": "prefixed", "contracted_rate": 12,
                   "market_rate": 10}),
        ),
        ("get_portfolio_risk", json!({})),
        ("rebalance_suggestion", json!({"contribution": 100000})),
        ("get_passive_income", json!({})),
        ("get_index_series", json!({"id": "ipca"})),
        (
            "get_inflation",
            json!({"deflator": "ipca", "from": "2026-01-01", "to": "2026-06-01"}),
        ),
        (
            "convert_currency",
            json!({"amount": 10000, "from": "USD", "to": "BRL"}),
        ),
        ("get_fundamentals", json!({"ticker": "PETR4"})),
        (
            "run_screen",
            json!({"filters": {"conditions": [{"field": "pl", "op": "lt", "value": 10}]}}),
        ),
        ("rank_assets", json!({"indicator": "dy"})),
        ("compare_assets", json!({"tickers": ["PETR4", "VALE3"]})),
        ("fair_price", json!({"ticker": "PETR4"})),
        ("list_corporate_documents", json!({})),
        ("get_fii_result", json!({"ticker": "XPTO11"})),
        ("list_physical_assets", json!({})),
        ("get_physical_asset", json!({"asset_id": "x"})),
    ];
    for (tool, args) in samples {
        let r = call(&client, tool, args).await;
        assert!(!text_of(&r).is_empty(), "{tool}");
        if r.is_error == Some(true) {
            let code = error_code(&r).unwrap();
            assert_ne!(code, "forbidden", "{tool}");
            assert_ne!(code, "internal", "{tool}: {}", text_of(&r));
        } else {
            assert!(r.structured_content.as_ref().unwrap().is_object(), "{tool}");
        }
    }
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn locked_vault_gives_friendly_error_and_audit_survives() {
    let v = vault();
    let client = connect(TreasuryMcp::new(v.api.clone(), "teste")).await;
    v.api.call("vault.lock", Value::Null).unwrap();

    let r = call(&client, "get_change_history", json!({})).await;
    assert_eq!(r.is_error, Some(true));
    assert_eq!(text_of(&r), locked_message());
    assert_eq!(error_code(&r).as_deref(), Some("vault_locked"));

    // Blocked, the audit stays pending (no parameters) and goes into the database later.
    let pending = v.paths.run_dir.join(audit::PENDING_FILE);
    let line = std::fs::read_to_string(&pending).unwrap();
    assert!(line.contains("get_change_history") && line.contains("locked"));
    assert!(!line.contains("params"));
    v.api
        .call("vault.unlock", json!({"password": PASSWORD}))
        .unwrap();
    let entries = audit_list(&v.api);
    let e = entries.iter().find(|e| !e.ok).expect("imported entry");
    assert_eq!(e.tool, "get_change_history");
    assert_eq!(e.error.as_deref(), Some("vault_locked"));
    assert!(!pending.exists());
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn read_only_client_cannot_reach_write_commands() {
    let v = vault();
    let server = TreasuryMcp::new(v.api.clone(), "teste");
    let actor = server.actor_for(Some("cli-teste"));
    assert_eq!(
        actor,
        Actor::Mcp {
            client: "cli-teste via teste".into(),
            read_only: true
        }
    );
    let fx = json!({"base": "USD", "quote": "BRL", "date": "2026-10-01", "rate": "5.1"});
    let f = server
        .call_api(&actor, "fx.set", fx.clone())
        .await
        .unwrap_err();
    assert_eq!(f.code, "forbidden");
    assert!(f.message.contains("somente leitura"));
    for cmd in [
        "fx.delete",
        "changes.undo",
        "vault.lock",
        "backup.create",
        "settings.set",
    ] {
        let f = server.call_api(&actor, cmd, json!({})).await.unwrap_err();
        assert_eq!(f.code, "forbidden", "{cmd}");
    }
    // Reading from the same module is allowed.
    assert!(server.call_api(&actor, "fx.list", json!({})).await.is_ok());
    // And nothing was written.
    let fx_rows = v.api.call("fx.list", json!({})).unwrap();
    assert_eq!(fx_rows, json!([]));
}

#[tokio::test]
async fn resources_and_prompts() {
    let v = vault();
    let client = connect(TreasuryMcp::new(v.api.clone(), "teste")).await;
    let templates = client.list_all_resource_templates().await.unwrap();
    let uris: Vec<&str> = templates.iter().map(|t| t.uri_template.as_str()).collect();
    assert_eq!(
        uris,
        [
            "treasury://summary/monthly/{yyyy-mm}",
            "treasury://budget/{yyyy-mm}"
        ]
    );
    let resources = client.list_all_resources().await.unwrap();
    assert_eq!(resources.len(), 2);

    let err = client
        .read_resource(ReadResourceRequestParams::new("treasury://budget/2026-13"))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("mês inválido"));
    assert!(
        client
            .read_resource(ReadResourceRequestParams::new("treasury://portfolio"))
            .await
            .is_err()
    );
    // Valid resource: data, or a readable error while the report does not exist yet.
    match client
        .read_resource(ReadResourceRequestParams::new(
            "treasury://summary/monthly/2026-10",
        ))
        .await
    {
        Ok(r) => assert_eq!(r.contents.len(), 1),
        Err(e) => assert!(!e.to_string().contains("permissão negada"), "{e}"),
    }
    assert!(
        audit_list(&v.api).iter().any(|e| e.tool == "resources/read"
            && e.params["uri"] == "treasury://summary/monthly/2026-10")
    );

    let prompts = client.list_all_prompts().await.unwrap();
    let titles: Vec<_> = prompts.iter().filter_map(|p| p.title.as_deref()).collect();
    assert_eq!(titles, ["Revisão mensal", "Plano para meta"]);
    let mut args = Map::new();
    args.insert("mes".into(), json!("2026-09"));
    let p = client
        .get_prompt(GetPromptRequestParams::new("revisao_mensal").with_arguments(args))
        .await
        .unwrap();
    let text = serde_json::to_string(&p.messages).unwrap();
    assert!(text.contains("2026-09"));
    assert!(text.contains("get_dashboard"));
    assert!(text.contains("centavos"));
    assert!(text.contains("não aconselhamento"));
    let p = client
        .get_prompt(GetPromptRequestParams::new("plano_para_meta"))
        .await
        .unwrap();
    assert!(
        serde_json::to_string(&p.messages)
            .unwrap()
            .contains("list_goals")
    );
    let mut bad = Map::new();
    bad.insert("mes".into(), json!("setembro"));
    assert!(
        client
            .get_prompt(GetPromptRequestParams::new("revisao_mensal").with_arguments(bad))
            .await
            .is_err()
    );
    client.cancel().await.unwrap();
}

/// Family in the US: the instructions, the tool summaries and the prompts speak in dollars,
/// even on values that come without a currency (month flow, passive income).
#[tokio::test]
async fn a_us_vault_talks_in_dollars() {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Arc::new(Api::new_unlocked_process(paths).unwrap());
    api.call(
        "vault.create",
        json!({"password": PASSWORD, "household_name": "Smith", "members": ["Ann"],
               "country": "US"}),
    )
    .unwrap();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Checking", "kind": "checking", "opening_date": "2026-01-01",
                   "opening_balance": 100_000}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    api.call(
        "transactions.create",
        json!({"account_id": acc, "date": "2026-09-05", "amount": 250_000,
               "description": "Salary"}),
    )
    .unwrap();
    allow_keyless(&api);
    let client = connect(TreasuryMcp::new(api.clone(), "teste")).await;
    let instructions = client
        .peer_info()
        .unwrap()
        .instructions
        .clone()
        .unwrap_or_default();
    assert!(
        instructions.contains("123456 with USD = USD 1.234,56")
            && instructions.contains("base currency is USD"),
        "{instructions}"
    );
    for (tool, args) in [
        (
            "get_monthly_flow",
            json!({"from_month": "2026-09", "to_month": "2026-09"}),
        ),
        ("get_passive_income", json!({})),
        ("get_portfolio", json!({})),
        ("get_net_worth", json!({})),
        ("get_dashboard", json!({})),
    ] {
        let r = call(&client, tool, args).await;
        assert_eq!(r.is_error, Some(false), "{tool}: {}", text_of(&r));
        let summary = r.content[0].as_text().unwrap().text.clone();
        assert!(
            summary.contains("USD") && !summary.contains("R$"),
            "{tool}: {summary}"
        );
    }
    let p = client
        .get_prompt(GetPromptRequestParams::new("plano_para_meta"))
        .await
        .unwrap();
    let text = serde_json::to_string(&p.messages).unwrap();
    assert!(
        text.contains("123456 com USD = USD 1.234,56") && !text.contains("R$"),
        "{text}"
    );
    // Without a known vault, the summary keeps the real (old vaults).
    let spec = tools::find("get_passive_income").unwrap();
    assert!(
        spec.summarize_in(&json!({"monthly_avg": 12345}), Some("EUR"))
            .contains("EUR 123,45")
    );
    assert!(
        spec.summarize(&json!({"monthly_avg": 12345}))
            .contains("R$ 123,45")
    );
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn unavailable_backend_explains_how_to_unlock() {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let msg = open_read_only_api(&paths).err().unwrap();
    assert!(msg.contains("Nenhum cofre"));
    let server = TreasuryMcp::unavailable(msg.clone(), "stdio", Some(paths.run_dir.clone()));
    let client = connect(server).await;
    assert_eq!(
        client.list_all_tools().await.unwrap().len(),
        EXPECTED_TOOLS.len()
    );
    let r = call(&client, "list_accounts", json!({})).await;
    assert_eq!(r.is_error, Some(true));
    assert_eq!(text_of(&r), msg);
    assert_eq!(error_code(&r).as_deref(), Some("unavailable"));
    // Pending audit, no data.
    let pending = std::fs::read_to_string(paths.run_dir.join(audit::PENDING_FILE)).unwrap();
    assert!(pending.contains("list_accounts"));
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn read_only_fallback_uses_keychain_without_taking_the_app_lock() {
    let v = vault();
    allow_keyless(&v.api);
    // Without keychain enabled: explains how to enable it.
    v.api.call("vault.lock", Value::Null).unwrap();
    let msg = open_read_only_api(&v.paths).err().unwrap();
    assert!(msg.contains("desbloqueio pelo keychain"));

    v.api
        .call("vault.unlock", json!({"password": PASSWORD}))
        .unwrap();
    v.api
        .call("vault.keychain_enable", json!({"password": PASSWORD}))
        .unwrap();
    v.api.call("vault.lock", Value::Null).unwrap();

    let ro = Arc::new(open_read_only_api(&v.paths).expect("opens in read mode"));
    // The app can open (exclusive lock free) with the MCP client connected.
    let app = Api::new(v.paths.clone()).expect("app opens with the read-only MCP connected");
    app.call("vault.unlock", json!({"password": PASSWORD}))
        .unwrap();

    let client = connect(TreasuryMcp::new(ro.clone(), "treasury mcp (app fechado)")).await;
    let r = call(&client, "get_change_history", json!({})).await;
    assert_eq!(r.is_error, Some(false), "{}", text_of(&r));
    assert_eq!(r.structured_content.unwrap()["count"], 1);
    // The read-only database does not write: the audit stays pending and the app imports.
    assert!(v.paths.run_dir.join(audit::PENDING_FILE).exists());
    let entries = audit_list(&app);
    let e = entries
        .iter()
        .find(|e| e.tool == "get_change_history")
        .expect("imported by the app");
    assert_eq!(e.client, "teste-contrato via treasury mcp (app fechado)");
    assert!(e.ok);
    client.cancel().await.unwrap();
}

/// With the app closed, `treasury mcp` only answers with the setting on and with a valid
/// API key (docs/local-security.md, item 2).
#[tokio::test]
async fn closed_app_needs_the_setting_and_a_valid_key() {
    let v = vault();
    v.api
        .call("vault.keychain_enable", json!({"password": PASSWORD}))
        .unwrap();
    let made = v
        .api
        .call(
            "mcp.key_create",
            json!({"name": "Cliente externo", "scopes": ["read"]}),
        )
        .unwrap();
    let key = made["key"].as_str().unwrap().to_string();
    let settings = v.api.call("mcp.server_settings", json!({})).unwrap();
    assert_eq!(settings["cli_when_closed"], false);
    v.api.call("vault.lock", Value::Null).unwrap();

    let ro: crate::SharedApi = Arc::new(open_read_only_api(&v.paths).unwrap());
    // Default: off, even with the right key.
    let msg = super::cli::closed_app_access(&ro, Some(&key))
        .err()
        .unwrap();
    assert!(msg.contains("Responder com o app fechado"), "{msg}");
    drop(ro);

    v.api
        .call("vault.unlock", json!({"password": PASSWORD}))
        .unwrap();
    let mut on = settings.clone();
    on["cli_when_closed"] = json!(true);
    // Even with "Require key" off, with the app closed the key remains mandatory.
    on["require_api_key"] = json!(false);
    v.api.call("mcp.set_server_settings", on).unwrap();
    v.api.call("vault.lock", Value::Null).unwrap();

    let ro: crate::SharedApi = Arc::new(open_read_only_api(&v.paths).unwrap());
    assert!(super::cli::closed_app_access(&ro, None).is_err());
    assert!(super::cli::closed_app_access(&ro, Some("trsk_falsa")).is_err());
    let grant = super::cli::closed_app_access(&ro, Some(&key)).unwrap();
    assert!(grant.has(super::keys::McpScope::Read));
}

#[test]
fn client_config_command() {
    let v = vault();
    let cfg = v
        .api
        .call(
            "mcp.client_config",
            json!({"command": "/Applications/Treasury.app/Contents/MacOS/treasury"}),
        )
        .unwrap();
    let server = &cfg["mcpServers"]["treasury"];
    assert_eq!(
        server["command"],
        "/Applications/Treasury.app/Contents/MacOS/treasury"
    );
    assert_eq!(server["args"], json!(["mcp"]));
    // Root outside the default goes in TREASURY_HOME.
    assert_eq!(
        server["env"]["TREASURY_HOME"],
        v.paths.root.to_string_lossy().as_ref()
    );
    let plain = client_config(std::path::Path::new("/usr/bin/treasury"), None);
    assert_eq!(
        plain,
        json!({"mcpServers": {"treasury": {"command": "/usr/bin/treasury", "args": ["mcp"]}}})
    );
    let reader = Actor::Mcp {
        client: "x".into(),
        read_only: true,
    };
    assert!(v.api.call_as(&reader, "mcp.audit_list", json!({})).is_ok());
}

#[tokio::test]
async fn write_tools_follow_settings_and_become_proposals() {
    let v = vault();
    allow_keyless(&v.api);
    let acc = v
        .api
        .call(
            "accounts.create",
            json!({"name": "Conta", "kind": "checking"}),
        )
        .unwrap()["id"]
        .clone();
    let client = connect(TreasuryMcp::new(v.api.clone(), "teste")).await;
    assert_eq!(
        client.list_all_tools().await.unwrap().len(),
        EXPECTED_TOOLS.len()
    );
    let args =
        json!({"account_id": acc, "date": "2026-10-01", "amount": -1500, "description": "Café"});
    // Off: the write tool is refused even if the client knows the name.
    let r = call(&client, "create_transaction", args.clone()).await;
    assert_eq!(error_code(&r).as_deref(), Some("forbidden"));
    assert!(text_of(&r).contains("Ajustes"));
    v.api
        .call("ai.settings.set", json!({"mcp_allow_writes": true}))
        .unwrap();
    let tools = client.list_all_tools().await.unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    assert_eq!(names.len(), EXPECTED_TOOLS.len() + WRITE_TOOLS.len());
    for (w, _) in WRITE_TOOLS {
        assert!(names.contains(&w), "{w}");
        let t = tools.iter().find(|t| t.name == w).unwrap();
        assert_eq!(t.annotations.as_ref().unwrap().read_only_hint, Some(false));
    }
    let r = call(&client, "create_transaction", args).await;
    assert_eq!(r.is_error, Some(false), "{}", text_of(&r));
    let data = r.structured_content.clone().unwrap();
    assert_eq!(data["status"], "pending_approval");
    assert!(text_of(&r).contains("aguarda a aprovação"));
    let p = v
        .api
        .call("ai.get_proposal", json!({"id": data["proposal_id"]}))
        .unwrap();
    assert_eq!(p["client"], "teste-contrato via teste");
    assert_eq!(
        v.api.call("transactions.query", json!({})).unwrap()["total"],
        0
    );
    client.cancel().await.unwrap();
}

/// Companion registration tools: each becomes a proposal with the policy level;
/// in autonomous mode only the reversible ones go straight through.
#[test]
fn register_tools_become_proposals_with_policy_levels() {
    let v = vault();
    let agent = Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    let run = |name: &str, args: Value| {
        let o = inproc::call_tool(&v.api, &agent, name, args, inproc::AGENT_ACCESS);
        assert!(!o.is_error, "{name}: {}", o.content);
        let data: Value = serde_json::from_str(o.content.split_once('\n').unwrap().1).unwrap();
        data
    };
    let level = |pid: &Value| {
        v.api.call("ai.get_proposal", json!({"id": pid})).unwrap()["level"]
            .as_str()
            .unwrap()
            .to_string()
    };
    // Bank and card in a single proposal (institution by name).
    let p = run(
        "create_account",
        json!({"name": "Nubank", "kind": "credit_card", "institution_name": "Nubank",
               "card": {"closing_day": 3, "due_day": 10, "credit_limit": 500000, "last4": "4321"}}),
    );
    assert_eq!(p["status"], "pending_approval");
    assert_eq!(level(&p["proposal_id"]), "reversible");
    assert_eq!(v.api.call("accounts.list", json!({})).unwrap(), json!([]));
    v.api
        .call("ai.approve", json!({"ids": [p["proposal_id"]]}))
        .unwrap();
    let accounts = v.api.call("accounts.list", json!({})).unwrap();
    let card = &accounts[0];
    assert_eq!(card["institution_name"], "Nubank");
    let inst = v
        .api
        .call("household.list_institutions", json!({}))
        .unwrap();
    assert_eq!(inst[0]["kind"], "issuer");
    // Changing an existing account is sensitive.
    let p = run(
        "update_account",
        json!({"id": card["id"], "card": {"closing_day": 5, "due_day": 12}}),
    );
    assert_eq!(level(&p["proposal_id"]), "sensitive");
    // Asset and fixed income.
    let p = run(
        "create_asset",
        json!({"name": "CDB Banco X", "class": "fixed_income",
               "fixed_income": {"kind": "cdb", "indexer": "cdi", "rate": "110",
                                "maturity_date": "2028-01-01"}}),
    );
    assert_eq!(level(&p["proposal_id"]), "reversible");
    // Autonomous mode: reversible ones applied directly; sensitive ones stay in the queue.
    v.api
        .call("ai.settings.set", json!({"autonomous": true}))
        .unwrap();
    let c = run("create_category", json!({"name": "Pet", "kind": "expense"}));
    assert_ne!(c["status"], "pending_approval", "{c}");
    assert!(
        v.api
            .call("categories.list", json!({}))
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["name"] == "Pet")
    );
    let inst_id = inst[0]["id"].clone();
    let u = run(
        "update_institution",
        json!({"id": inst_id, "website": "https://nubank.com.br"}),
    );
    assert_ne!(u["status"], "pending_approval");
    let p = run(
        "update_account",
        json!({"id": card["id"], "name": "Nubank Ultravioleta"}),
    );
    assert_eq!(
        p["status"], "pending_approval",
        "sensitive is never autonomous"
    );
    // Invalid inputs do not reach the core.
    let o = inproc::call_tool(
        &v.api,
        &agent,
        "create_account",
        json!({"name": "Cartão", "kind": "credit_card"}),
        inproc::AGENT_ACCESS,
    );
    assert!(o.is_error && o.content.contains("card"), "{}", o.content);
    let s = run("get_suggestions", json!({"screen": "contas"}));
    assert!(s["suggestions"].is_array());
}

#[test]
fn every_tool_schema_is_valid_for_all_providers() {
    let all: Vec<(&str, &str)> = EXPECTED_TOOLS
        .iter()
        .chain(WRITE_TOOLS.iter())
        .chain(MEMORY_TOOLS.iter())
        .chain(DEV_TOOLS.iter())
        .chain(LONGEVITY_TOOLS.iter())
        .copied()
        .collect();
    assert_eq!(tools::TOOLS.len(), all.len());
    for (spec, (name, cmd)) in tools::TOOLS.iter().zip(all) {
        assert_eq!((spec.name, spec.command), (name, cmd));
        let s = spec.input_schema();
        assert_eq!(s["type"], "object", "{name}");
        assert_eq!(s["additionalProperties"], false, "{name}");
        for (p, d) in s["properties"].as_object().unwrap() {
            assert!(
                !d["description"].as_str().unwrap_or_default().is_empty()
                    || d.get("$ref").is_some(),
                "{name}.{p} has no description"
            );
        }
        let g = crate::ai::providers::gemini_schema(&s).to_string();
        assert!(
            !g.contains("$ref") && !g.contains("additionalProperties"),
            "{name}: {g}"
        );
        // Writes only through the policy (proposals).
        let is_write = WRITE_TOOLS.iter().any(|(n, _)| *n == name) || name == "delete_dev_feedback";
        assert_eq!(
            crate::api::policy::level(cmd).is_some() || crate::api::policy::is_own_proposal(cmd),
            is_write,
            "{name}"
        );
        let is_memory = MEMORY_TOOLS.iter().any(|(n, _)| *n == name);
        assert_eq!(
            crate::api::policy::is_agent_memory(cmd) || crate::api::policy::is_agent_context(cmd),
            is_memory,
            "{name}"
        );
        assert_eq!(
            spec.group() == tools::Group::AgentMemory,
            is_memory,
            "{name}"
        );
        let is_dev = DEV_TOOLS.iter().any(|(n, _)| *n == name);
        assert_eq!(spec.group() == tools::Group::DevFeedback, is_dev, "{name}");
        let is_longevity = LONGEVITY_TOOLS.iter().any(|(n, _)| *n == name);
        assert_eq!(
            spec.group() == tools::Group::Longevity,
            is_longevity,
            "{name}"
        );
        if is_longevity {
            assert!(spec.is_read_only(), "{name}");
        }
        if is_dev {
            let read = DEV_READS.contains(&name);
            assert_eq!(
                crate::api::policy::is_dev_feedback(cmd),
                !read && name != "delete_dev_feedback",
                "{name}"
            );
            assert_eq!(spec.is_read_only(), read, "{name}");
        }
    }
    let defs = inproc::tool_defs(inproc::AGENT_ACCESS);
    assert_eq!(
        defs.len(),
        tools::TOOLS.len() - DEV_TOOLS.len() - LONGEVITY_TOOLS.len()
    );
    let defs = inproc::tool_defs(tools::Access {
        dev: true,
        ..inproc::AGENT_ACCESS
    });
    assert_eq!(defs.len(), tools::TOOLS.len() - LONGEVITY_TOOLS.len());
    let defs = inproc::tool_defs(tools::Access {
        dev: true,
        longevity: true,
        ..inproc::AGENT_ACCESS
    });
    assert_eq!(defs.len(), tools::TOOLS.len());
}

#[cfg(unix)]
mod socket_tests {
    use std::os::unix::fs::PermissionsExt;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    fn mode(p: &std::path::Path) -> u32 {
        std::fs::metadata(p).unwrap().permissions().mode() & 0o777
    }

    #[tokio::test]
    async fn handshake_accepts_right_token_and_closes_wrong_one() {
        let v = vault();
        let run_dir = v.paths.run_dir.clone();
        // Default: without an API key, the right token is not enough.
        {
            let server = start_socket_server(v.api.clone(), &run_dir).await.unwrap();
            // The refusal arrives with the reason, for `treasury mcp` to relay to the client.
            match connect_socket(&run_dir, "sem-chave").await {
                Err(crate::error::Error::Forbidden(why)) => {
                    assert_eq!(why, crate::i18n::t("mcp-chave-exigida"))
                }
                other => panic!("{:?}", other.map(|_| ())),
            }
            drop(server);
        }
        allow_keyless(&v.api);
        let server = start_socket_server(v.api.clone(), &run_dir).await.unwrap();
        assert_eq!(mode(server.socket_path()), 0o600);
        assert_eq!(mode(server.token_path()), 0o600);
        assert_eq!(mode(&run_dir), 0o700);
        let token = std::fs::read_to_string(server.token_path()).unwrap();
        assert_eq!(token.len(), 43);

        // Right token: full MCP through the socket.
        let (rd, wr) = connect_socket(&run_dir, "cli-de-teste").await.unwrap();
        let client = client_info().serve((rd, wr)).await.unwrap();
        assert_eq!(
            client.list_all_tools().await.unwrap().len(),
            EXPECTED_TOOLS.len()
        );
        let r = call(&client, "get_change_history", json!({})).await;
        assert_eq!(r.is_error, Some(false), "{}", text_of(&r));
        let e = audit_list(&v.api)
            .into_iter()
            .find(|e| e.tool == "get_change_history")
            .unwrap();
        assert_eq!(e.client, "teste-contrato via cli-de-teste");

        // New client in this vault: notice with the program and the PID (docs/local-security.md,
        // item 14), until dismissed.
        let settings = || -> api::McpServerSettings {
            serde_json::from_value(v.api.call("mcp.server_settings", json!({})).unwrap()).unwrap()
        };
        let mut notices = Vec::new();
        for _ in 0..100 {
            notices = settings().new_clients;
            if !notices.is_empty() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        assert_eq!(notices.len(), 1, "{notices:?}");
        assert_eq!(notices[0].client, "cli-de-teste");
        assert!(notices[0].pid.is_some() && notices[0].program.is_some());
        v.api.call("mcp.dismiss_client_notices", json!({})).unwrap();
        assert!(settings().new_clients.is_empty());

        // Locked vault: the socket keeps accepting, the tools refuse.
        v.api.call("vault.lock", Value::Null).unwrap();
        let r = call(&client, "get_change_history", json!({})).await;
        assert_eq!(text_of(&r), locked_message());
        client.cancel().await.unwrap();

        // Wrong token: connection closed without a reply.
        let mut s = tokio::net::UnixStream::connect(server.socket_path())
            .await
            .unwrap();
        s.write_all(b"{\"token\":\"errado\",\"client\":\"x\"}\n")
            .await
            .unwrap();
        let mut buf = Vec::new();
        let n = tokio::time::timeout(std::time::Duration::from_secs(5), s.read_to_end(&mut buf))
            .await
            .unwrap()
            .unwrap_or(0);
        assert_eq!(n, 0, "no byte after a wrong token");

        // A line that is not a handshake: also closed.
        let mut s = tokio::net::UnixStream::connect(server.socket_path())
            .await
            .unwrap();
        s.write_all(b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\"}\n")
            .await
            .unwrap();
        let mut buf = Vec::new();
        let _ = s.read_to_end(&mut buf).await;
        assert!(buf.is_empty());

        // Token replaced on disk (another session): connect_socket fails.
        std::fs::write(server.token_path(), "outro-token").unwrap();
        assert!(connect_socket(&run_dir, "x").await.is_err());

        // Job's agent token: gets write and memory; the calls go to the job.
        v.api
            .call("vault.unlock", json!({"password": PASSWORD}))
            .unwrap();
        let rt = v.api.ai_runtime();
        let job = rt.new_job(
            crate::ai::jobs::JobKind::Chat,
            crate::ai::routing::AiTask::Chat,
            None,
        );
        let agent_token = rt.grant("agente (CLI de teste)".into(), &job.id).unwrap();
        let alt = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(server.socket_path(), alt.path().join(SOCKET_FILE)).unwrap();
        std::fs::write(alt.path().join(TOKEN_FILE), &agent_token).unwrap();
        let (rd, wr) = connect_socket(alt.path(), "treasury mcp").await.unwrap();
        let agent = client_info().serve((rd, wr)).await.unwrap();
        let names: Vec<String> = agent
            .list_all_tools()
            .await
            .unwrap()
            .into_iter()
            .map(|t| t.name.to_string())
            .collect();
        assert!(names.iter().any(|n| n == "create_goal"));
        assert!(names.iter().any(|n| n == "remember_procedure"));
        let r = call(
            &agent,
            "create_goal",
            json!({"name": "Reserva", "kind": "emergency", "target_amount": 1000000}),
        )
        .await;
        assert_eq!(r.structured_content.unwrap()["status"], "pending_approval");
        let events = job.events_after(0);
        assert!(events.iter().any(|e| matches!(
            &e.event,
            crate::ai::jobs::JobEventKind::Proposal { tool, .. } if tool == "create_goal"
        )));
        let pending = v
            .api
            .call("ai.list_proposals", json!({"status": "pending"}))
            .unwrap();
        assert_eq!(
            pending[0]["client"],
            "teste-contrato via agente (CLI de teste)"
        );
        agent.cancel().await.unwrap();
        // Job finished: the token stops being valid.
        job.finish(crate::ai::jobs::JobStatus::Done, None, None);
        assert!(connect_socket(alt.path(), "x").await.is_err());

        let sock = server.socket_path().to_path_buf();
        server.shutdown().await;
        assert!(!sock.exists());
        assert!(!run_dir.join(TOKEN_FILE).exists());
    }

    #[tokio::test]
    async fn socket_path_too_long_is_a_clear_error() {
        let v = vault();
        let long = v.paths.root.join("x".repeat(120)).join("run");
        let err = start_socket_server(v.api.clone(), &long)
            .await
            .err()
            .expect("path too long");
        assert!(err.to_string().contains("longo demais"), "{err}");
    }

    #[tokio::test]
    async fn orphan_socket_is_replaced_and_live_one_is_not() {
        let v = vault();
        allow_keyless(&v.api);
        let run_dir = v.paths.run_dir.clone();
        // Orphan socket: a socket file with no process serving it.
        let orphan = std::os::unix::net::UnixListener::bind(run_dir.join(SOCKET_FILE)).unwrap();
        drop(orphan);
        assert!(run_dir.join(SOCKET_FILE).exists());
        let server = start_socket_server(v.api.clone(), &run_dir).await.unwrap();
        // With a live server, a second one does not start.
        assert!(matches!(
            start_socket_server(v.api.clone(), &run_dir).await,
            Err(crate::Error::AlreadyOpen)
        ));
        assert!(connect_socket(&run_dir, "x").await.is_ok());
        drop(server);
        assert!(!run_dir.join(SOCKET_FILE).exists());
    }
}

/// The texts the tools return (summaries, fixed notes, validation errors and the expected
/// format) follow the installation language; field names, ids, ISO dates and JSON stay as
/// they are. In pt-BR, the usual text.
#[test]
fn tool_texts_follow_the_installation_locale() {
    use crate::i18n::{Locale, set_current};
    let spec = |name: &str| tools::find(name).unwrap();
    let invalid = |name: &str, raw: Value| spec(name).prepare(raw).err().unwrap();
    let net_worth = json!({
        "net": 123456, "assets": 200000, "liabilities": 76544,
        "currency": "BRL", "as_of": "2026-10-19",
    });
    let fundamentals = json!({
        "subject": {"ticker": "PETR4"},
        "indicators": [
            {"key": "pl", "value": 5.5}, {"key": "dy", "value": 0.0612}, {"key": "roe", "value": null},
        ],
    });
    let receipt = json!({
        "items": [{}], "parsed": {"store_name": "Mercado X", "paid": 1500},
        "tax": 300, "tax_printed": true, "duplicate": "abc",
    });

    set_current(Locale::En);
    let cents = "Money amounts in cents (smallest currency unit).";
    assert_eq!(
        spec("list_accounts").summarize(&json!({"accounts": [{}], "count": 1})),
        format!("1 account. {cents}")
    );
    assert_eq!(
        spec("list_accounts").summarize(&json!({"count": 3})),
        format!("3 accounts. {cents}")
    );
    assert_eq!(
        spec("get_net_worth").summarize(&net_worth),
        format!(
            "Net worth as of 2026-10-19: R$1,234.56 (assets R$2,000.00, liabilities R$765.44). \
             {cents}"
        )
    );
    assert_eq!(
        spec("get_iptu").summarize(&json!([])),
        format!("No IPTU bill recorded for this property. {cents}")
    );
    assert_eq!(
        spec("get_iptu").summarize(&json!([{}, {}])),
        format!("2 IPTU bills. {cents}")
    );
    // As it really arrives: the list without `list_key` comes wrapped in `{"result": [...]}`.
    let iptu = spec("get_iptu");
    assert_eq!(
        iptu.summarize(&iptu.structured(json!([{}, {}]))),
        format!("2 IPTU bills. {cents}")
    );
    // Negative balance already on the first forecast day: "já hoje", not "fica negativa em hoje".
    let forecast = json!({"from": "2026-10-20", "to": "2027-01-20", "currency": "BRL",
        "total_start": -100, "total_end": -200, "total_min": -200, "total_min_date": "2027-01-15",
        "accounts": [{"name": "Conta", "first_negative": "2026-10-20"},
                     {"name": "Outra", "first_negative": "2026-11-05"}]});
    let f = spec("get_balance_forecast").summarize(&forecast);
    assert!(
        f.contains("Conta already today") && f.contains("Outra on 2026-11-05"),
        "{f}"
    );
    let returns = spec("get_returns");
    let one = returns.summarize(&json!({"months": [{}], "cumulative": [0.0]}));
    assert!(!one.contains("1 months"), "{one}");
    let capital = spec("get_capital_shares");
    assert_eq!(
        capital.summarize(&capital.structured(Value::Null)),
        capital.summarize(&Value::Null)
    );
    let scores = spec("get_portfolio_scores");
    assert!(
        scores
            .summarize(&scores.structured(json!([{"score": 70.0}, {"score": null}])))
            .starts_with("2 "),
        "{}",
        scores.summarize(&scores.structured(json!([{"score": 70.0}])))
    );
    assert_eq!(
        spec("preview_receipt").summarize(&receipt),
        format!(
            "Receipt from Mercado X: 1 item, paid R$15.00, tax R$3.00 (printed on the \
             receipt), already imported. {cents}"
        )
    );
    let research = spec("get_fundamentals").summarize(&fundamentals);
    assert!(
        research.starts_with(
            "PETR4: 2 of 3 indicators available; P/L 5.50; DY 6.12%. Indicators: percentages \
             as fractions (0.06 = 6%)"
        ),
        "{research}"
    );
    let e = invalid("query_transactions", json!({"date_from": "ontem"}));
    assert!(
        e.starts_with(
            "`date_from` must be a date YYYY-MM-DD. Format: no required fields; optional `"
        ),
        "{e}"
    );
    assert!(
        e.ends_with(". Each field's description is in the tool's schema."),
        "{e}"
    );
    let e = invalid("explain_value", json!({"metric": "spending"}));
    assert!(
        e.starts_with("`from` is required for this measure. Format: required `metric`"),
        "{e}"
    );
    let e = invalid("list_recurring", json!({"x": 1}));
    assert!(
        e.ends_with(". This tool takes no parameters: call it with {}."),
        "{e}"
    );
    assert_eq!(
        invalid_params("list_accounts", "x"),
        "Invalid parameters for list_accounts: x"
    );

    set_current(Locale::Es);
    assert_eq!(
        spec("list_accounts").summarize(&json!({"count": 1})),
        "1 cuenta. Valores monetarios en centavos (menor unidad de la moneda)."
    );

    set_current(Locale::PtBr);
    let centavos = "Valores monetários em centavos (menor unidade da moeda).";
    assert_eq!(
        spec("list_accounts").summarize(&json!({"count": 1})),
        format!("1 conta. {centavos}")
    );
    assert_eq!(
        spec("list_accounts").summarize(&json!({"count": 0})),
        format!("0 contas. {centavos}")
    );
    assert_eq!(
        spec("get_net_worth").summarize(&net_worth),
        format!(
            "Patrimônio líquido em 2026-10-19: R$ 1.234,56 (ativos R$ 2.000,00, passivos \
             R$ 765,44). {centavos}"
        )
    );
    assert_eq!(
        spec("get_iptu").summarize(&json!([{}, {}])),
        format!("2 carnês de IPTU. {centavos}")
    );
    assert_eq!(
        spec("preview_receipt").summarize(&receipt),
        format!(
            "Cupom de Mercado X: 1 item(ns), pago R$ 15,00, imposto R$ 3,00 (impresso na \
             nota), já importado. {centavos}"
        )
    );
    let research = spec("get_fundamentals").summarize(&fundamentals);
    assert!(
        research.starts_with(
            "PETR4: 2 de 3 indicadores disponíveis; P/L 5,50; DY 6,12%. Indicadores: \
             percentuais em fração (0,06 = 6%)"
        ),
        "{research}"
    );
    let e = invalid("query_transactions", json!({"date_from": "ontem"}));
    assert!(
        e.starts_with(
            "`date_from` deve ser uma data AAAA-MM-DD. Formato: nenhum campo obrigatório; \
             opcionais `"
        ),
        "{e}"
    );
    assert!(
        e.ends_with(". A descrição de cada campo está no esquema da ferramenta."),
        "{e}"
    );
    let e = invalid("list_recurring", json!({"x": 1}));
    assert!(
        e.ends_with(". Esta ferramenta não recebe parâmetros: chame com {}."),
        "{e}"
    );
}

/// Summary of `get_spend_groups` in the installation language, with the months' plural and the
/// catalog group names.
#[test]
fn spend_groups_summary_follows_the_locale() {
    use crate::i18n::{Locale, set_current};
    let spec = tools::find("get_spend_groups").unwrap();
    let g = |group: &str, amount: i64, share: Value| json!({"group": group, "amount": amount, "share_of_income": share, "top": []});
    let report = |months: usize, income: i64, shares: [Value; 4]| {
        let [a, b, c, d] = shares;
        json!({
            "currency": "USD",
            "months": vec![json!({}); months],
            "total": {"month": "total", "income": income, "spending": 900_000, "groups": [
                g("essential", 500_000, a), g("debt", 0, b),
                g("future", 150_000, c), g("non_essential", 250_000, d),
            ]},
            "targets": {"essential": 0.5, "debt": null, "future": 0.2, "non_essential": 0.3},
        })
    };
    let cents = "Money amounts in cents (smallest currency unit).";
    set_current(Locale::En);
    let two = spec.summarize(&report(
        2,
        1_000_000,
        [json!(0.5), json!(0.0), json!(0.15), json!(0.25)],
    ));
    let one = spec.summarize(&report(
        1,
        0,
        [Value::Null, Value::Null, Value::Null, Value::Null],
    ));
    let empty = spec.summarize(&json!({}));
    set_current(Locale::PtBr);
    assert_eq!(
        two,
        format!(
            "Where the income went in 2 months (income $10,000.00): Essentials $5,000.00 (50% of \
             income); Debts $0.00 (0% of income); Future $1,500.00 (15% of income); \
             Non-essential $2,500.00 (25% of income). {cents}"
        )
    );
    assert_eq!(
        one,
        format!(
            "Where the income went in 1 month (income $0.00): Essentials $5,000.00; Debts $0.00; \
             Future $1,500.00; Non-essential $2,500.00. {cents}"
        )
    );
    assert_eq!(empty, format!("Spending groups calculated. {cents}"));
    let pt = spec.summarize(&report(
        1,
        0,
        [Value::Null, Value::Null, Value::Null, Value::Null],
    ));
    assert!(
        pt.starts_with(
            "Para onde foi a renda em 1 mês (receitas USD 0,00): Essenciais USD 5.000,00;"
        ),
        "{pt}"
    );
}

/// A validation error already ending in a period does not come out with ".." (catalog closes it).
#[test]
fn invalid_errors_end_with_a_single_period() {
    let v = vault();
    let acc = v
        .api
        .call(
            "accounts.create",
            json!({"name": "Conta", "kind": "checking"}),
        )
        .unwrap();
    let agent = Actor::Mcp {
        client: "agente".into(),
        read_only: true,
    };
    let o = inproc::call_tool(
        &v.api,
        &agent,
        "list_fgts_entries",
        json!({"account_id": acc["id"]}),
        inproc::AGENT_ACCESS,
    );
    assert!(o.is_error, "{}", o.content);
    assert!(!o.content.contains(".."), "{}", o.content);
    assert!(o.content.trim_end().ends_with('.'), "{}", o.content);
    let o = inproc::call_tool(
        &v.api,
        &agent,
        "project_fgts",
        json!({}),
        inproc::AGENT_ACCESS,
    );
    assert!(o.content.contains("`age`"), "{}", o.content);
}

#[test]
fn brazil_only_tools_hide_outside_brazil() {
    let abroad = tools::Access {
        outside_brazil: true,
        ..inproc::AGENT_ACCESS
    };
    let home: Vec<&str> = tools::visible(inproc::AGENT_ACCESS)
        .map(|t| t.command)
        .collect();
    let away: Vec<&str> = tools::visible(abroad).map(|t| t.command).collect();
    // In Brazil, everything; outside, no FGTS, no IR with DARF, no IPTU, no brokerage notes.
    assert!(home.iter().any(|c| c.starts_with("fgts.")));
    assert!(home.contains(&"physical.iptu_list"));
    for c in &away {
        assert!(
            !c.starts_with("fgts.") && !c.starts_with("tax.") && !c.starts_with("notes."),
            "{c}"
        );
    }
    assert!(!away.contains(&"goals.inss_estimate"));
    assert!(!away.contains(&"debt.market_rates"));
    // The rest stays: transactions, portfolio, taxes paid in the year.
    assert!(away.contains(&"transactions.query"));
    assert!(away.contains(&"tax_burden.estimate"));
    assert!(
        home.len() - away.len() >= 20,
        "{} → {}",
        home.len(),
        away.len()
    );
}

#[test]
fn create_account_checks_the_retirement_plan_before_proposing() {
    use super::inputs::ToolInput;
    use super::inputs_register::CreateAccountInput;
    let input = |v: serde_json::Value| -> CreateAccountInput { serde_json::from_value(v).unwrap() };
    let bad =
        input(serde_json::json!({"name": "x", "kind": "retirement", "retirement_plan": "foo"}));
    let err = bad.to_api_args().unwrap_err();
    assert!(err.contains("us_401k") && err.contains("pt_ppr"), "{err}");
    let wrong_kind =
        input(serde_json::json!({"name": "x", "kind": "checking", "retirement_plan": "us_ira"}));
    assert!(wrong_kind.to_api_args().is_err());
    let coverage = input(
        serde_json::json!({"name": "x", "kind": "retirement", "retirement_plan": "us_hsa", "retirement_coverage": "x"}),
    );
    assert!(coverage.to_api_args().is_err());
    let ok = input(
        serde_json::json!({"name": "HSA", "kind": "retirement", "retirement_plan": "us_hsa", "retirement_coverage": "family"}),
    );
    assert!(ok.to_api_args().is_ok());
}

#[test]
fn period_fields_accept_from_to_and_the_long_names() {
    // The canonical name of each tool and `from`/`to` (or the long names) give the same
    // arguments to the core.
    let args = |tool: &str, v: Value| tools::find(tool).unwrap().prepare(v).unwrap().args;
    let flow = args(
        "get_monthly_flow",
        json!({"from_month": "2026-01", "to_month": "2026-06"}),
    );
    assert_eq!(flow["from_month"], "2026-01");
    assert_eq!(
        args(
            "get_monthly_flow",
            json!({"from": "2026-01", "to": "2026-06"})
        ),
        flow
    );
    let groups = args(
        "get_spend_groups",
        json!({"from": "2026-01", "to": "2026-06"}),
    );
    assert_eq!(
        args(
            "get_spend_groups",
            json!({"from_month": "2026-01", "to_month": "2026-06"})
        ),
        groups
    );
    let txns = args(
        "query_transactions",
        json!({"from": "2026-01-01", "to": "2026-01-31"}),
    );
    assert_eq!(
        (txns["date_from"].as_str(), txns["date_to"].as_str()),
        (Some("2026-01-01"), Some("2026-01-31"))
    );
    let spend = args(
        "spending_summary",
        json!({"start_date": "2026-01-01", "end_date": "2026-01-31"}),
    );
    assert_eq!(spend["from"], "2026-01-01");
    for tool in [
        "list_trades",
        "get_returns",
        "list_sales",
        "list_income_events",
    ] {
        assert!(
            tools::find(tool)
                .unwrap()
                .prepare(json!({"from": "2026-01-01", "to": "2026-02-01"}))
                .is_ok()
                || tools::find(tool)
                    .unwrap()
                    .prepare(json!({"from": "2026-01", "to": "2026-02"}))
                    .is_ok(),
            "{tool}"
        );
    }
    // Both names together are the same field twice: clear error, not a silent one.
    assert!(
        tools::find("get_monthly_flow")
            .unwrap()
            .prepare(json!({"from": "2026-01", "from_month": "2026-02"}))
            .is_err()
    );
}

/// CLI as engine: the vault's extra MCPs arrive through Treasury's MCP (job token), and the
/// call waits for the person's confirmation in the conversation.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cli_agent_gets_extra_mcp_tools_through_treasury_with_confirmation() {
    if std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }
    let v = vault();
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("fake_mcp.py");
    std::fs::write(&script, crate::ai::mcp_extra::tests::FAKE_SERVER).unwrap();
    v.api
        .call(
            "ai.mcp_extra.save",
            json!({"name": "celular", "label": "Celular", "transport": "stdio",
                   "command": "python3", "args": [script.to_string_lossy()],
                   "env": [{"key": "FAKE_TOKEN", "value": "tok-exemplo-1234", "secret": true}]}),
        )
        .unwrap();
    let rt = v.api.ai_runtime();
    let job = rt.new_job(
        crate::ai::jobs::JobKind::Chat,
        crate::ai::routing::AiTask::Chat,
        None,
    );
    // Opening the sessions creates its own runtime: outside the test's runtime.
    let (api, job_id) = (v.api.clone(), job.id.clone());
    std::thread::spawn(move || {
        let servers = api
            .with_session(|s, _| crate::ai::mcp_extra::enabled(s.db.conn(), s.db.key()))
            .unwrap();
        api.ai_runtime()
            .set_extras(&job_id, crate::ai::mcp_extra::Toolbox::open(&servers));
    })
    .join()
    .unwrap();
    let client = connect(TreasuryMcp::for_agent(
        v.api.clone(),
        "treasury mcp",
        &job.id,
    ))
    .await;
    let names: Vec<String> = client
        .list_all_tools()
        .await
        .unwrap()
        .into_iter()
        .map(|t| t.name.to_string())
        .collect();
    assert!(names.iter().any(|n| n == "celular-echo"), "{names:?}");
    // The person allows it through the conversation card.
    let (api, j) = (v.api.clone(), job.clone());
    let approver = std::thread::spawn(move || {
        for _ in 0..200 {
            let pending = j.events_after(0).into_iter().find_map(|e| match e.event {
                crate::ai::jobs::JobEventKind::ExternalApproval {
                    request_id, tool, ..
                } => Some((request_id, tool)),
                _ => None,
            });
            if let Some((id, tool)) = pending {
                assert_eq!(tool, "echo");
                api.call(
                    "ai.mcp_extra.decide",
                    json!({"request_id": id, "allow": true}),
                )
                .unwrap();
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        panic!("no confirmation request");
    });
    let r = call(&client, "celular-echo", json!({"text": "oi"})).await;
    approver.join().unwrap();
    assert!(
        text_of(&r).contains("eco: oi tok-exemplo-1234"),
        "{}",
        text_of(&r)
    );
    assert!(!r.is_error.unwrap_or(false));
    // Outside client (no agent token) does not see the extra MCPs.
    let outside = connect(TreasuryMcp::new(v.api.clone(), "externo")).await;
    let names: Vec<String> = outside
        .list_all_tools()
        .await
        .unwrap()
        .into_iter()
        .map(|t| t.name.to_string())
        .collect();
    assert!(!names.iter().any(|n| n.starts_with("celular-")));
    let rt = v.api.ai_runtime();
    let id = job.id.clone();
    let api = v.api.clone();
    std::thread::spawn(move || api.ai_runtime().drop_extras(&id))
        .join()
        .unwrap();
    let _ = rt;
}

/// The conversation's agent (job token) asks with a form and gets the answer; an outside
/// client does not have the tool.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_asks_the_user_with_a_form_and_gets_the_answer() {
    let v = vault();
    let rt = v.api.ai_runtime();
    let job = rt.new_job(
        crate::ai::jobs::JobKind::Chat,
        crate::ai::routing::AiTask::Chat,
        None,
    );
    let client = connect(TreasuryMcp::for_agent(
        v.api.clone(),
        "treasury mcp",
        &job.id,
    ))
    .await;
    let names: Vec<String> = client
        .list_all_tools()
        .await
        .unwrap()
        .into_iter()
        .map(|t| t.name.to_string())
        .collect();
    assert!(names.iter().any(|n| n == "ask_user"));
    let (api, j) = (v.api.clone(), job.clone());
    let answerer = std::thread::spawn(move || {
        for _ in 0..200 {
            let pending = j.events_after(0).into_iter().find_map(|e| match e.event {
                crate::ai::jobs::JobEventKind::UserQuestion {
                    request_id,
                    question,
                } => Some((request_id, question)),
                _ => None,
            });
            if let Some((id, q)) = pending {
                assert_eq!(q.recommended, Some(0));
                // Two options without multiple choice: refused; one: accepted.
                assert!(
                    api.call(
                        "ai.question.answer",
                        json!({"request_id": id, "selected": ["Conta corrente", "Poupança"]}),
                    )
                    .is_err()
                );
                api.call(
                    "ai.question.answer",
                    json!({"request_id": id, "selected": [], "other": "a conta do banco B"}),
                )
                .unwrap();
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        panic!("no question");
    });
    let r = call(
        &client,
        "ask_user",
        json!({"question": "Qual conta?", "options": ["Conta corrente", "Poupança"], "recommended": 0}),
    )
    .await;
    answerer.join().unwrap();
    let text = text_of(&r);
    assert!(text.contains("Other: \"a conta do banco B\""), "{text}");
    assert!(job.events_after(0).iter().any(|e| matches!(
        &e.event,
        crate::ai::jobs::JobEventKind::UserAnswer {
            answer: Some(_),
            ..
        }
    )));
    let outside = connect(TreasuryMcp::new(v.api.clone(), "externo")).await;
    let names: Vec<String> = outside
        .list_all_tools()
        .await
        .unwrap()
        .into_iter()
        .map(|t| t.name.to_string())
        .collect();
    assert!(!names.iter().any(|n| n == "ask_user"));
}

/// Label as a concentration group via the MCP: becomes a proposal; nothing changes before approval.
#[test]
fn concentration_group_by_mcp_waits_for_approval() {
    let v = vault();
    let agent = Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    let a = v
        .api
        .call(
            "assets.create",
            json!({"ticker": "AAAA3", "name": "Empresa A", "class": "stock_br"}),
        )
        .unwrap()["id"]
        .clone();
    v.api
        .call(
            "assets.labels_set",
            json!({"asset_id": a, "labels": ["Grupo X"]}),
        )
        .unwrap();
    let o = inproc::call_tool(
        &v.api,
        &agent,
        "tag_as_concentration_group",
        json!({"name": "Grupo X", "concentration_limit_pct": 25}),
        inproc::AGENT_ACCESS,
    );
    assert!(!o.is_error, "{}", o.content);
    let data: Value = serde_json::from_str(o.content.split_once('\n').unwrap().1).unwrap();
    assert_eq!(data["status"], "pending_approval");
    let label = |api: &Api| api.call("assets.labels_list", json!({})).unwrap()[0].clone();
    assert_eq!(label(&v.api)["is_concentration_group"], false);
    assert!(label(&v.api)["concentration_limit_pct"].is_null());
    v.api
        .call("ai.approve", json!({"ids": [data["proposal_id"]]}))
        .unwrap();
    assert_eq!(label(&v.api)["is_concentration_group"], true);
    assert_eq!(label(&v.api)["concentration_limit_pct"], 25.0);
}

#[tokio::test]
async fn inbox_files_reach_the_tools_by_id_or_allowed_path() {
    use base64::Engine;
    let v = vault();
    allow_keyless(&v.api);
    let inbox = tempfile::tempdir().unwrap();
    v.api
        .call(
            "settings.set",
            json!({"inbox_folder": inbox.path().display().to_string()}),
        )
        .unwrap();
    let ofx = "OFXHEADER:100\nDATA:OFXSGML\n<OFX><BANKMSGSRSV1><STMTTRNRS><STMTRS><CURDEF>BRL\
        <BANKACCTFROM><BANKID>0260<ACCTID>98765</BANKACCTFROM><BANKTRANLIST>\
        <STMTTRN><TRNTYPE>DEBIT<DTPOSTED>20261001<TRNAMT>-10.00<FITID>X1<MEMO>MERCADO EXEMPLO\
        </STMTTRN></BANKTRANLIST></STMTRS></STMTTRNRS></BANKMSGSRSV1></OFX>";
    let file = inbox.path().join("extrato.ofx");
    std::fs::write(&file, ofx).unwrap();
    // File already written (the app waits a second so as not to read a half copy).
    std::fs::File::options()
        .write(true)
        .open(&file)
        .unwrap()
        .set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(10))
        .unwrap();

    let client = connect(TreasuryMcp::new(v.api.clone(), "agente-externo")).await;
    let r = call(&client, "list_inbox", json!({})).await;
    assert_eq!(r.is_error, Some(false), "{}", text_of(&r));
    let items = r.structured_content.as_ref().unwrap()["items"].clone();
    assert_eq!(items.as_array().unwrap().len(), 1, "{items}");
    assert_eq!(items[0]["detected_kind"], "statement");
    assert_eq!(items[0]["status"], "pending");
    // Left the folder: the content stays in the vault.
    assert!(!file.exists());
    let id = items[0]["id"].as_str().unwrap().to_string();

    // `inbox_id` becomes the same `files` as the base64 upload.
    let base64 = base64::engine::general_purpose::STANDARD.encode(ofx);
    let (by_id, ids) = inbox_files::resolve(
        &v.api,
        "import_statement_file",
        json!({"inbox_id": [id.clone()], "account_id": "c"}),
    )
    .unwrap();
    assert_eq!(ids, std::slice::from_ref(&id));
    assert_eq!(by_id["files"][0]["content_base64"], base64.as_str());
    assert_eq!(by_id["files"][0]["filename"], "extrato.ofx");
    assert!(by_id.get("inbox_id").is_none());
    assert!(
        tools::find("import_statement_file")
            .unwrap()
            .prepare(by_id)
            .is_ok()
    );
    // A tool that does not read files and a path outside the allowed folders: refused.
    assert!(inbox_files::resolve(&v.api, "list_accounts", json!({"inbox_id": [id]})).is_err());
    let outside = tempfile::tempdir().unwrap();
    let elsewhere = outside.path().join("nota.pdf");
    std::fs::write(&elsewhere, b"%PDF-1.4").unwrap();
    let err = inbox_files::resolve(
        &v.api,
        "preview_brokerage_notes",
        json!({"path": elsewhere.display().to_string(), "account_id": "c"}),
    )
    .unwrap_err();
    assert!(err.contains("fora das pastas permitidas"), "{err}");
}

/// Basket inflation through the MCP: in a vault without repeated purchases the tool answers without
/// a number, and the summary puts the basket next to the official index when both exist.
#[test]
fn basket_inflation_tool_reads_the_family_basket() {
    let v = vault();
    let reader = Actor::Mcp {
        client: "cli-teste".into(),
        read_only: true,
    };
    let o = inproc::call_tool(
        &v.api,
        &reader,
        "get_basket_inflation",
        json!({"months": 6}),
        tools::Access::default(),
    );
    assert!(!o.is_error, "{}", o.content);
    assert!(
        o.note.summary.contains("compras repetidas suficientes"),
        "{}",
        o.note.summary
    );
    let data: Value = serde_json::from_str(o.content.split_once('\n').unwrap().1).unwrap();
    assert_eq!(data["enough"], false);
    assert_eq!(data["months"].as_array().unwrap().len(), 6);

    let spec = tools::find("get_basket_inflation").unwrap();
    assert!(spec.is_read_only());
    let full = json!({
        "enough": true, "months": [{}, {}, {}], "change": 0.132, "official_change": 0.079,
        "official_name": "IPCA · Alimentação no domicílio",
        "same_basket_now": 126_800, "same_basket_year_ago": 112_000,
    });
    let text = spec.summarize(&full);
    assert!(
        text.starts_with(
            "Sua cesta em 3 meses: 13,2%; IPCA · Alimentação no domicílio: 7,9%. As compras dos \
             últimos 3 meses custaram R$ 1.268,00; há um ano custariam R$ 1.120,00."
        ),
        "{text}"
    );
    let alone = json!({"enough": true, "months": [{}, {}, {}], "change": -0.01});
    let text = spec.summarize(&alone);
    assert!(
        text.starts_with(
            "Sua cesta em 3 meses: -1,0% (sem o índice oficial de alimentos para comparar)."
        ),
        "{text}"
    );
}

/// D18: what the interface reads (`useCore('<module>', '<action>')`), an MCP client reads too.
/// Each exception says why it stays out; a new interface command without a tool or exception
/// breaks this test.
#[test]
fn every_read_shown_by_the_interface_has_a_tool() {
    use std::collections::BTreeSet;
    // Technical modules: settings, vault, maintenance, support, terms and the MCP itself.
    const TECHNICAL: &[&str] = &[
        "app.",
        "backup.",
        "country.",
        "database.",
        "mail.",
        "mcp.",
        "notify.",
        "profile.",
        "settings.",
        "sources.",
        "support.",
        "system.",
        "terms.",
    ];
    // (command, reason).
    const EXCEPTIONS: &[(&str, &str)] = &[
        ("accounts.staleness_limits", "ajuste"),
        ("health.get_settings", "ajuste"),
        ("longevity.settings", "ajuste"),
        ("tax_burden.settings", "ajuste"),
        ("categories.missing_defaults", "sugestão de ajustes"),
        ("food.categories", "opções de formulário"),
        ("food.profile", "opções de formulário"),
        ("food.stores", "opções de formulário"),
        ("food.purchase_candidates", "opções de formulário"),
        ("food.merge_suggestions", "revisão de cadastro"),
        ("fx.exchange_quote", "opções de formulário"),
        ("household.banks", "opções de formulário"),
        ("import.list_presets", "opções de formulário"),
        ("market.exchanges", "opções de formulário"),
        ("market.data_sources", "ajuste"),
        ("physical.fipe_options", "opções de formulário"),
        ("physical.iptu_matches", "opções de formulário"),
        ("debt.payment_candidates", "opções de formulário"),
        ("recurrences.preview", "prévia de formulário"),
        ("retirement.plans", "opções de formulário"),
        (
            "tax.annual_compute",
            "recalcula o formulário da declaração editado na tela",
        ),
        ("tax_burden.categories", "tabelas de referência"),
        ("tax_burden.dataset", "tabelas de referência"),
        ("tax_burden.datasets", "tabelas de referência"),
        (
            "transactions.operation_link_preview",
            "prévia de formulário",
        ),
        ("transactions.transfer_match", "opções de formulário"),
        ("transactions.list_attachments", "arquivos: read_document"),
        // Already covered by another tool.
        ("cards.get_statement", "list_card_statements"),
        (
            "cards.list_installment_plans",
            "get_card_future_commitments",
        ),
        ("cards.loyalty_entries", "list_loyalty_programs"),
        ("categories.tree", "list_categories"),
        ("categories.usage", "list_categories"),
        ("changes.get", "get_change_history"),
        ("debt.get", "list_loans"),
        ("fx.list", "get_fx_reference_rate"),
        ("household.get", "list_members"),
        ("household.members_overview", "list_members"),
        ("import.list_batches", "get_change_history"),
        ("longevity.stages", "get_family_outlook"),
        ("market.quotes", "get_price_history"),
        ("pension.get", "list_pension_payments"),
        ("physical.summary", "list_physical_assets"),
        ("projects.get", "list_projects"),
        ("receipts.get", "list_purchases"),
        ("receipts.list", "list_purchases"),
        ("retirement.status", "get_retirement_accounts"),
        // City list for choosing the city: reading the data is `tax_burden.city`.
        ("tax_burden.cities", "get_city_tax_revenue"),
        ("transactions.get", "query_transactions"),
        ("transactions.list_tags", "query_transactions"),
        // Health data (allergies and restrictions per member): outside the MCP until the
        // scope (docs/decisions.md, D39).
        ("food.diet_list", "dado de saúde"),
        ("food.health_summary", "dado de saúde"),
    ];
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/desktop/src");
    let re = regex::Regex::new(r"(?s)useCore<[^(]*?>\(\s*'([a-z_]+)',\s*'([a-z_.]+)'").unwrap();
    let mut used = BTreeSet::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for e in std::fs::read_dir(&dir).unwrap() {
            let p = e.unwrap().path();
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            if p.is_dir() {
                if name != "mock" && name != "node_modules" {
                    stack.push(p);
                }
            } else if (name.ends_with(".ts") || name.ends_with(".tsx")) && !name.contains(".test.")
            {
                let text = std::fs::read_to_string(&p).unwrap();
                for c in re.captures_iter(&text) {
                    used.insert(format!("{}.{}", &c[1], &c[2]));
                }
            }
        }
    }
    assert!(
        used.len() > 100,
        "the interface search found only {}",
        used.len()
    );
    let tools: BTreeSet<&str> = tools::TOOLS.iter().map(|t| t.command).collect();
    let missing: Vec<&String> = used
        .iter()
        .filter(|c| !tools.contains(c.as_str()))
        .filter(|c| !TECHNICAL.iter().any(|p| c.starts_with(p)))
        .filter(|c| !EXCEPTIONS.iter().any(|(e, _)| e == c))
        .collect();
    assert!(
        missing.is_empty(),
        "interface reads with no MCP tool or exception: {missing:?}"
    );
    // Old exception (the interface no longer reads it, or the tool already exists) also fails.
    let stale: Vec<&str> = EXCEPTIONS
        .iter()
        .map(|(e, _)| *e)
        .filter(|e| !used.contains(*e) || tools.contains(e))
        .collect();
    assert!(
        stale.is_empty(),
        "exceptions that no longer apply: {stale:?}"
    );
}
