//! Invariantes de dinheiro nos cofres de demonstração (docs/invariants.md): contas que
//! precisam bater entre si em qualquer família, conferidas nos cinco perfis do assistente e em
//! mais de um país. Teste por função pega o erro que alguém pensou em testar; o invariante pega
//! o "o número não bate" que chega à família.
//!
//! Cada invariante compara dois caminhos do núcleo para o mesmo número. Quando um quebra, o
//! erro diz o perfil, o país, o mês e os dois valores.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::api::{Actor, Api};
use crate::paths::TreasuryPaths;
use crate::reports::engine::{month_dates, shift_month};
use crate::time::{month_of, today_local};

/// Os cinco perfis do assistente (features/demo/presets.ts), com 6 meses de histórico.
fn presets() -> Vec<(&'static str, Value)> {
    vec![
        (
            "comfortable",
            json!({"adults": [{"name": "Ana", "income": 1_200_000}, {"name": "Bruno", "income": 850_000}],
                "children": ["Clara"], "housing": "mortgage", "housing_amount": 420_000, "condo": 90_000,
                "utilities": 90_000, "groceries": 220_000, "transport": "car", "transport_amount": 130_000,
                "school_per_child": 240_000, "health": 180_000, "leisure": 150_000, "subscriptions": 15_000,
                "shopping": 90_000, "card_limit": 2_500_000, "reserve": 6_000_000, "reserve_monthly": 150_000,
                "stocks": 8_000_000, "fiis": 4_000_000, "fixed_income": 5_000_000, "invest_monthly": 250_000,
                "pension": 3_000_000, "pension_monthly": 80_000}),
        ),
        (
            "low_income",
            json!({"adults": [{"name": "Maria", "income": 190_000}, {"name": "João", "income": 160_000, "variable": true}],
                "children": ["Davi", "Elisa"], "benefit": 60_000, "housing": "rent", "housing_amount": 90_000,
                "utilities": 42_000, "groceries": 145_000, "transport": "public", "transport_amount": 38_000,
                "leisure": 15_000, "subscriptions": 4_000, "shopping": 28_000, "card_limit": 120_000,
                "reserve": 150_000, "reserve_monthly": 3_000}),
        ),
        (
            "struggling",
            json!({"adults": [{"name": "Rosa", "income": 230_000}, {"name": "Carlos", "income": 90_000, "variable": true}],
                "children": ["Davi", "Elisa"], "housing": "rent", "housing_amount": 110_000, "utilities": 38_000,
                "groceries": 130_000, "transport": "public", "transport_amount": 30_000, "leisure": 6_000,
                "subscriptions": 5_000, "shopping": 20_000, "card_limit": 180_000, "personal_loan": 45_000,
                "personal_loan_left": 18, "overdraft_limit": 150_000, "late_bills": true, "card_revolving": true}),
        ),
        (
            "single",
            json!({"adults": [{"name": "Lucas", "income": 420_000}], "housing": "rent",
                "housing_amount": 150_000, "condo": 35_000, "utilities": 28_000, "groceries": 70_000,
                "transport": "app", "transport_amount": 45_000, "leisure": 70_000, "subscriptions": 9_000,
                "shopping": 40_000, "card_limit": 500_000, "reserve": 300_000, "reserve_monthly": 30_000,
                "fiis": 200_000, "fixed_income": 500_000, "invest_monthly": 30_000}),
        ),
        (
            "retiree",
            json!({"adults": [{"name": "Helena", "income": 380_000, "retired": true}],
                "other_income": 150_000, "housing": "own", "condo": 60_000, "utilities": 42_000,
                "groceries": 110_000, "transport": "none", "health": 130_000, "leisure": 40_000,
                "subscriptions": 6_000, "shopping": 20_000, "card_limit": 400_000, "reserve": 4_000_000,
                "reserve_monthly": 20_000, "fixed_income": 8_000_000}),
        ),
    ]
}

/// Cofre de demonstração do perfil, no país.
fn demo(kind: &str, profile: &Value, country: &str) -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new(TreasuryPaths::with_root(dir.path().join(".t")).unwrap()).unwrap();
    let mut p = profile.clone();
    p["kind"] = json!(kind);
    p["months"] = json!(6);
    p["bank"] = json!("Banco");
    p["country"] = json!(country);
    api.call(
        "vaults.new_demo",
        json!({"name": format!("{country} {kind}"), "household_name": "Demo", "profile": p}),
    )
    .unwrap_or_else(|e| panic!("{country} {kind}: {e}"));
    (dir, api)
}

fn call(api: &Api, cmd: &str, args: Value) -> Value {
    api.call(cmd, args).unwrap_or_else(|e| panic!("{cmd}: {e}"))
}

fn int(v: &Value, what: &str) -> i64 {
    v.as_i64().unwrap_or_else(|| panic!("{what}: {v}"))
}

/// Linhas por tabela do cofre: para ver se uma leitura gravou alguma coisa.
fn row_counts(api: &Api) -> BTreeMap<String, i64> {
    api.with_session(|s, _| {
        let conn = s.db.conn();
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        )?;
        let tables: Vec<String> = stmt
            .query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        let mut out = BTreeMap::new();
        for t in tables {
            let n: i64 =
                conn.query_row(&format!("SELECT count(*) FROM \"{t}\""), [], |r| r.get(0))?;
            out.insert(t, n);
        }
        Ok(out)
    })
    .unwrap()
}

/// Leituras do dia a dia da família, todas sem rede: nenhuma pode gravar no cofre.
const READS: &[(&str, &str)] = &[
    ("reports.dashboard", "{}"),
    ("reports.net_worth", "{}"),
    ("reports.insights", "{}"),
    ("reports.upcoming", "{}"),
    ("accounts.list", "{}"),
    ("categories.list", "{}"),
    ("household.get", "{}"),
    ("household.list_members", "{}"),
    ("goals.list", "{}"),
    ("goals.emergency_reserve", "{}"),
    ("health.score", "{}"),
    ("forecast.balance", "{}"),
    ("forecast.daily_allowance", "{}"),
    ("subscriptions.list", "{}"),
    ("installments.detected", "{}"),
    ("debt.list", "{}"),
    ("ai.suggestions", "{}"),
    ("ai.alerts.list", "{}"),
    ("attention.list", "{}"),
    ("attention.coverage", "{}"),
    ("attention.next_step", "{}"),
    ("attention.month_close", "{}"),
];

fn check(kind: &str, country: &str, api: &Api) {
    let who = format!("{country}/{kind}");
    let current = month_of(today_local());

    // 1. O mês pelo fluxo, pelo painel e pela soma das categorias é o mesmo número.
    for back in 1..=3 {
        let month = shift_month(&current, -back).unwrap();
        let (from, to) = month_dates(&month).unwrap();
        let flow = call(
            api,
            "reports.month_flow",
            json!({"from_month": month, "to_month": month}),
        );
        let flow = &flow.as_array().unwrap_or_else(|| panic!("{who}: {flow}"))[0];
        let dash = call(api, "reports.dashboard", json!({"month": month}));
        for side in ["income", "expense"] {
            let by_flow = int(&flow[side], side);
            assert_eq!(
                int(&dash["flow"][side], side),
                by_flow,
                "{who} {month}: {side} do painel × fluxo do mês"
            );
            let spending = call(
                api,
                "reports.spending",
                json!({"from": from, "to": to, "kind": side, "group_by": "parent_category"}),
            );
            assert_eq!(
                int(&spending["total"], "total"),
                by_flow,
                "{who} {month}: total de {side} por categoria × fluxo do mês"
            );
            let groups: i64 = spending["groups"]
                .as_array()
                .unwrap_or_else(|| panic!("{who}: {spending}"))
                .iter()
                .map(|g| int(&g["amount"], "amount"))
                .sum();
            assert_eq!(
                groups, by_flow,
                "{who} {month}: soma das categorias de {side} × fluxo do mês"
            );
        }
        // O que sobrou é receita menos despesa.
        assert_eq!(
            int(&flow["net"], "net"),
            int(&flow["income"], "income") - int(&flow["expense"], "expense"),
            "{who} {month}: sobra do mês"
        );
    }

    // 2. Patrimônio: líquido = ativos − passivos, e as partes informadas cabem no todo.
    let nw = call(api, "reports.net_worth", json!({}));
    let (assets, liabilities, net) = (
        int(&nw["assets"], "assets"),
        int(&nw["liabilities"], "liabilities"),
        int(&nw["net"], "net"),
    );
    assert_eq!(net, assets - liabilities, "{who}: patrimônio líquido");
    assert!(
        liabilities >= 0 && assets >= 0,
        "{who}: sinais do patrimônio {nw}"
    );
    assert!(
        int(&nw["investments"], "investments") + int(&nw["physical"], "physical") <= assets,
        "{who}: carteira e bens cabem nos ativos"
    );
    assert!(
        int(&nw["debts"], "debts") <= liabilities,
        "{who}: contratos cabem nos passivos"
    );
    let dash = call(api, "reports.dashboard", json!({}));
    assert_eq!(
        int(&dash["net_worth"]["net"], "net"),
        net,
        "{who}: patrimônio do painel × patrimônio"
    );

    // 3. Por membro: as colunas somam o total da família.
    let month = shift_month(&current, -1).unwrap();
    let (from, to) = month_dates(&month).unwrap();
    let members = call(api, "reports.by_member", json!({"from": from, "to": to}));
    let columns: i64 = members["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| int(&m["amount"], "amount"))
        .sum();
    assert_eq!(
        columns,
        int(&members["total"], "total"),
        "{who} {month}: soma dos membros × total da família"
    );

    // 4. A previsão de saldo começa no saldo de hoje de cada conta.
    let forecast = call(api, "forecast.balance", json!({}));
    let accounts = call(api, "accounts.list", json!({}));
    let balance_of = |id: &str| {
        accounts
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["id"] == id)
            .map(|a| int(&a["balance"], "balance"))
    };
    for a in forecast["accounts"].as_array().unwrap() {
        let id = a["account_id"].as_str().unwrap();
        assert_eq!(
            Some(int(&a["start"], "start")),
            balance_of(id),
            "{who}: a previsão de {} começa no saldo de hoje",
            a["name"]
        );
    }

    // 5. A lista "Para resolver" conta o que mostra, sem item repetido.
    let att = call(api, "attention.list", json!({}));
    let items = att["items"].as_array().unwrap();
    assert_eq!(
        int(&att["counts"]["total"], "total"),
        items.len() as i64,
        "{who}"
    );
    let mut ids: Vec<&str> = items.iter().map(|i| i["id"].as_str().unwrap()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(
        ids.len(),
        items.len(),
        "{who}: item repetido em Para resolver"
    );
    let step = call(api, "attention.next_step", json!({}));
    assert!(
        (1..=6).contains(&int(&step["rung"], "rung")),
        "{who}: {step}"
    );

    // 6. Leitura não grava: um cliente só de leitura chama as consultas do dia a dia e nenhuma
    // tabela do cofre muda de tamanho.
    let reader = Actor::Mcp {
        client: "invariantes".into(),
        read_only: true,
    };
    let budget = json!({"month": current}).to_string();
    let reads = READS
        .iter()
        .copied()
        .chain([("budget.get", budget.as_str())]);
    for (cmd, args) in reads {
        let before = row_counts(api);
        let args: Value = serde_json::from_str(args).unwrap();
        api.call_as(&reader, cmd, args)
            .unwrap_or_else(|e| panic!("{who}: {cmd}: {e}"));
        let after = row_counts(api);
        let changed: Vec<String> = after
            .iter()
            .filter(|(t, n)| before.get(*t) != Some(n))
            .map(|(t, n)| format!("{t}: {} → {n}", before.get(t).copied().unwrap_or(0)))
            .collect();
        assert!(
            changed.is_empty(),
            "{who}: {cmd} gravou no cofre: {changed:?}"
        );
    }
}

#[test]
fn money_invariants_hold_for_every_demo_profile_in_brazil() {
    for (kind, profile) in presets() {
        let (_d, api) = demo(kind, &profile, "BR");
        check(kind, "BR", &api);
    }
}

#[test]
fn money_invariants_hold_in_other_countries_and_currencies() {
    let all = presets();
    for (country, locale, pick) in [
        ("US", crate::i18n::Locale::En, 0),
        ("PT", crate::i18n::Locale::Es, 2),
    ] {
        crate::i18n::set_current(locale);
        let (kind, profile) = &all[pick];
        let (_d, api) = demo(kind, profile, country);
        check(kind, country, &api);
        crate::i18n::set_current(crate::i18n::Locale::PtBr);
    }
}
