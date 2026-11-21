use serde_json::json;

use crate::api::Api;
use crate::paths::TreasuryPaths;

fn api() -> (tempfile::TempDir, Api, String) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
    )
    .unwrap();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Conta", "kind": "checking", "opening_date": "2025-01-01"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    (dir, api, acc)
}

#[test]
fn loan_lifecycle_schedule_reconcile_and_net_worth() {
    let (_d, api, acc) = api();
    let l = api
        .call(
            "debt.create",
            json!({"name": "Financiamento do carro", "kind": "vehicle", "principal": 6_000_000, "annual_rate": "12.682503",
                   "system": "price", "contract_date": "2025-12-15", "first_due_date": "2026-01-15", "term_months": 48,
                   "debit_account_id": acc}),
        )
        .unwrap();
    let id = l["id"].as_str().unwrap().to_string();
    let sched = api.call("debt.schedule", json!({"id": id})).unwrap();
    assert_eq!(sched.as_array().unwrap().len(), 48);
    let pmt = sched[0]["total"].as_i64().unwrap();
    assert!(
        (pmt - 158_002).abs() <= 2,
        "Price de 60 mil a 1% a.m. em 48x ≈ 1.580,02: {pmt}"
    );
    // Pagamento da 1ª parcela no extrato concilia sozinho.
    api.call("transactions.create", json!({"account_id": acc, "date": "2026-01-16", "amount": -pmt, "description": "Parcela carro"})).unwrap();
    let r = api.call("debt.reconcile", json!({"id": id})).unwrap();
    assert!(r["matched"].as_i64().unwrap() >= 1);
    let v = api.call("debt.get", json!({"id": id})).unwrap();
    assert!(v["paid_count"].as_u64().unwrap() >= 1);
    assert!(v["interest_paid"].as_i64().unwrap() > 0);
    // Patrimônio inclui o saldo devedor como passivo.
    let nw = api
        .call("reports.net_worth", json!({"as_of": "2026-01-20"}))
        .unwrap();
    assert_eq!(nw["debts"], sched[0]["balance_after"]);
    assert!(nw["liabilities"].as_i64().unwrap() >= nw["debts"].as_i64().unwrap());
}

#[test]
fn prepayment_and_portability_simulations() {
    let (_d, api, acc) = api();
    let l = api
        .call(
            "debt.create",
            json!({"name": "Imóvel", "kind": "mortgage", "principal": 30_000_000, "annual_rate": "10", "system": "sac",
                   "contract_date": "2026-01-01", "first_due_date": "2026-02-01", "term_months": 360, "insurance_monthly": 9_000,
                   "debit_account_id": acc}),
        )
        .unwrap();
    let id = l["id"].as_str().unwrap();
    let term = api.call("debt.simulate_prepayment", json!({"loan_id": id, "date": "2026-06-05", "amount": 3_000_000, "mode": "reduce_term"})).unwrap();
    let inst = api.call("debt.simulate_prepayment", json!({"loan_id": id, "date": "2026-06-05", "amount": 3_000_000, "mode": "reduce_installment"})).unwrap();
    assert!(term["months_after"].as_u64().unwrap() < 360);
    assert_eq!(inst["months_after"], 360);
    assert!(term["interest_saved"].as_i64().unwrap() > inst["interest_saved"].as_i64().unwrap());
    assert!(
        inst["next_payment_after"].as_i64().unwrap()
            < inst["next_payment_before"].as_i64().unwrap()
    );
    let port = api
        .call(
            "debt.simulate_portability",
            json!({"loan_id": id, "annual_rate": "8", "costs": 200_000}),
        )
        .unwrap();
    assert!(
        port["savings"].as_i64().unwrap() > 0,
        "taxa menor economiza: {port}"
    );
    // Registrar a amortização extra recalcula o cronograma e lança a saída na conta.
    api.call(
        "debt.add_extra_payment",
        json!({"loan_id": id, "date": "2026-06-05", "amount": 3_000_000, "mode": "reduce_term"}),
    )
    .unwrap();
    let after = api.call("debt.schedule", json!({"id": id})).unwrap();
    assert_eq!(
        after.as_array().unwrap().len() as u64,
        term["months_after"].as_u64().unwrap()
    );
    let bal = api.call("accounts.get", json!({"id": acc})).unwrap()["balance"]
        .as_i64()
        .unwrap();
    assert_eq!(bal, -3_000_000);
    // Desfazer remove a amortização e volta o cronograma.
    let ch = api.call("changes.list", json!({"limit": 1})).unwrap();
    api.call("changes.undo", json!({"id": ch[0]["id"]}))
        .unwrap();
    assert_eq!(
        api.call("debt.schedule", json!({"id": id}))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        360
    );
}

#[test]
fn validation_errors() {
    let (_d, api, _acc) = api();
    let base = json!({"name": "X", "kind": "personal", "principal": 100_000, "annual_rate": "20", "system": "price",
                      "contract_date": "2026-02-01", "first_due_date": "2026-01-01", "term_months": 12});
    assert!(
        api.call("debt.create", base.clone()).is_err(),
        "1ª parcela antes do contrato"
    );
    let mut b = base.clone();
    b["first_due_date"] = json!("2026-03-01");
    b["grace_months"] = json!(12);
    assert!(
        api.call("debt.create", b).is_err(),
        "carência igual ao prazo"
    );
}

#[test]
fn extra_payment_without_txn() {
    let (_d, api, acc) = api();
    // Amortizar com FGTS (sem lançamento na conta de débito) reduzindo o prazo.
    let l = api
        .call(
            "debt.create",
            json!({"name": "Apartamento", "kind": "mortgage", "principal": 30_000_000, "annual_rate": "9.5", "system": "sac",
                   "indexer": "tr", "contract_date": "2026-01-10", "first_due_date": "2026-02-10", "term_months": 360,
                   "debit_account_id": acc}),
        )
        .unwrap();
    let id = l["id"].as_str().unwrap();
    api.call(
        "debt.add_extra_payment",
        json!({"loan_id": id, "date": "2026-04-20", "amount": 5_000_000, "mode": "reduce_term", "without_txn": true}),
    )
    .unwrap();
    let bal = api.call("accounts.get", json!({"id": acc})).unwrap()["balance"]
        .as_i64()
        .unwrap();
    assert_eq!(bal, 0, "FGTS não sai da conta de débito");
    let n = api
        .call("debt.schedule", json!({"id": id}))
        .unwrap()
        .as_array()
        .unwrap()
        .len();
    assert!(n < 360, "prazo reduzido: {n}");
}

#[test]
fn rate_from_installment_and_contract_number() {
    let (_d, api, _acc) = api();
    // 60 mil em 48x de 1.580,02 na Price ≈ 1% a.m. (12,68% a.a.).
    let s = api
        .call(
            "debt.solve_rate",
            json!({"principal": 6_000_000, "installment_amount": 158_002, "term_months": 48, "system": "price"}),
        )
        .unwrap();
    let monthly: f64 = s["monthly_rate"].as_str().unwrap().parse().unwrap();
    assert!((monthly - 1.0).abs() < 0.001, "{s}");
    let annual: f64 = s["annual_rate"].as_str().unwrap().parse().unwrap();
    assert!((annual - 12.6825).abs() < 0.01, "{s}");

    // Sem taxa, com a parcela (seguro de 30,00 incluído): o cronograma reproduz a parcela.
    let l = api
        .call(
            "debt.create",
            json!({"name": "CDC", "contract_number": "  0012345-6  ", "kind": "personal", "principal": 6_000_000,
                   "annual_rate": "", "installment_amount": 161_002, "insurance_monthly": 3_000, "system": "price",
                   "contract_date": "2025-12-15", "first_due_date": "2026-01-15", "term_months": 48}),
        )
        .unwrap();
    assert_eq!(l["contract_number"], "0012345-6");
    let id = l["id"].as_str().unwrap().to_string();
    let sched = api.call("debt.schedule", json!({"id": id})).unwrap();
    let pmt = sched[0]["total"].as_i64().unwrap();
    assert!((pmt - 161_002).abs() <= 2, "parcela do cronograma {pmt}");

    // SAC: a primeira parcela define a taxa (saldo ÷ prazo + saldo × taxa).
    let sac = api
        .call(
            "debt.solve_rate",
            json!({"principal": 12_000_000, "installment_amount": 220_000, "term_months": 120, "system": "sac"}),
        )
        .unwrap();
    let m: f64 = sac["monthly_rate"].as_str().unwrap().parse().unwrap();
    assert!((m - 1.0).abs() < 0.001, "{sac}");

    // Parcela que não cobre o valor financiado é recusada com explicação.
    let e = api
        .call(
            "debt.solve_rate",
            json!({"principal": 6_000_000, "installment_amount": 100_000, "term_months": 48, "system": "price"}),
        )
        .unwrap_err();
    assert_eq!(e.code(), "invalid");

    // O número do contrato pode ser trocado ou removido depois.
    let v = api
        .call("debt.update", json!({"id": id, "contract_number": "999"}))
        .unwrap();
    assert_eq!(v["contract_number"], "999");
    let v = api
        .call("debt.update", json!({"id": id, "contract_number": null}))
        .unwrap();
    assert!(v["contract_number"].is_null());
}

/// CDI diário constante (% a.d.) em todos os dias úteis do intervalo.
fn seed_daily(api: &Api, series: &str, from: &str, to: &str, daily: &str) {
    use crate::finance::calendar::is_business_day;
    let (mut d, end) = (
        crate::time::parse_date(from).unwrap(),
        crate::time::parse_date(to).unwrap(),
    );
    let v: rust_decimal::Decimal = daily.parse().unwrap();
    let mut vals = vec![];
    while d <= end {
        if is_business_day(d) {
            vals.push((d, v, false));
        }
        d = d.succ_opt().unwrap();
    }
    api.with_session(|s, _| crate::indices::upsert_values(s.db.conn(), series, &vals, "teste"))
        .unwrap();
}

#[test]
fn cdi_plus_spread_accrues_business_days_per_period() {
    use crate::finance::calendar::is_business_day;
    use rust_decimal::Decimal;
    use rust_decimal::prelude::*;
    let (_d, api, _acc) = api();
    seed_daily(&api, "cdi", "2026-01-01", "2026-03-31", "0.05501");
    let l = api
        .call(
            "debt.create",
            json!({"name": "CCB capital de giro", "kind": "other", "principal": 10_000_000, "annual_rate": "2",
                   "indexer": "cdi", "system": "sac", "contract_date": "2026-01-15",
                   "first_due_date": "2026-02-15", "term_months": 12}),
        )
        .unwrap();
    assert_eq!(l["indexer"], "cdi");
    assert_eq!(l["index_pct"], "100");
    assert_eq!(l["spread_mode"], "exp252");
    let id = l["id"].as_str().unwrap().to_string();
    let sched = api.call("debt.schedule", json!({"id": id})).unwrap();
    let rows = sched.as_array().unwrap();
    assert_eq!(rows.len(), 12);
    // 15/02/2026 é domingo e 16–17/02 são Carnaval: vence na quarta, 18/02.
    assert_eq!(rows[0]["due_date"], "2026-02-18");
    let start = crate::time::parse_date("2026-01-15").unwrap();
    let end = crate::time::parse_date("2026-02-18").unwrap();
    let mut du = 0;
    let mut x = start;
    while x < end {
        if is_business_day(x) {
            du += 1;
        }
        x = x.succ_opt().unwrap();
    }
    assert_eq!(rows[0]["business_days"], du);
    // Fator = (1,0005501)^du (8 casas) × 1,02^(du/252) (9 casas), com 9 casas.
    let fdi = (0..du)
        .fold(Decimal::ONE, |a, _| {
            (a * Decimal::new(10005501, 7)).round_dp_with_strategy(16, RoundingStrategy::ToZero)
        })
        .round_dp(8);
    let sp = crate::finance::pow_dec(Decimal::new(102, 2), Decimal::from(du) / Decimal::from(252))
        .round_dp(9);
    let factor = (fdi * sp).round_dp(9);
    let got: Decimal = rows[0]["index_factor"].as_str().unwrap().parse().unwrap();
    assert_eq!(got, factor);
    let interest = (Decimal::from(10_000_000) * (factor - Decimal::ONE))
        .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
        .to_i64()
        .unwrap();
    assert_eq!(rows[0]["interest"].as_i64().unwrap(), interest);
    // SAC: amortização constante; com CDI publicado até março, a 1ª parcela é oficial e as
    // de abril em diante ficam estimadas (última taxa conhecida).
    assert_eq!(rows[0]["amortization"], rows[5]["amortization"]);
    assert_eq!(rows[0]["estimated"], false);
    assert_eq!(rows[11]["estimated"], true);
    assert_eq!(rows[11]["balance_after"], 0);
}

#[test]
fn postfixed_bullet_percent_of_selic_and_validation() {
    let (_d, api, _acc) = api();
    seed_daily(&api, "selic", "2026-01-01", "2026-12-31", "0.05");
    let l = api
        .call(
            "debt.create",
            json!({"name": "Bullet", "kind": "other", "principal": 5_000_000, "annual_rate": "0",
                   "indexer": "selic", "index_pct": "110", "system": "bullet", "contract_date": "2026-03-02",
                   "first_due_date": "2026-04-01", "term_months": 6}),
        )
        .unwrap();
    let id = l["id"].as_str().unwrap().to_string();
    let rows = api.call("debt.schedule", json!({"id": id})).unwrap();
    let rows = rows.as_array().unwrap();
    assert_eq!(rows.len(), 6);
    // Só juros até a última, que leva o principal inteiro.
    assert!(
        rows[..5]
            .iter()
            .all(|r| r["amortization"] == 0 && r["balance_after"] == 5_000_000)
    );
    assert_eq!(rows[5]["amortization"], 5_000_000);
    // 110% da Selic em 22 du (02/03 a 01/04): fator diário 1 + 0,05% × 1,1.
    let f: f64 = rows[0]["index_factor"].as_str().unwrap().parse().unwrap();
    assert!((f - 1.00055f64.powi(22)).abs() < 1e-8, "{f}");
    assert_eq!(rows[0]["business_days"], 22);

    // A parcela não define a taxa de um pós (varia com o índice).
    let e = api
        .call(
            "debt.create",
            json!({"name": "X", "kind": "other", "principal": 5_000_000, "annual_rate": "",
                   "installment_amount": 100_000, "indexer": "cdi", "system": "price",
                   "contract_date": "2026-03-02", "first_due_date": "2026-04-01", "term_months": 6}),
        )
        .unwrap_err();
    assert_eq!(e.code(), "invalid");
    // Contrato pré antigo continua igual (mensal, sem fator por período).
    let pre = api
        .call(
            "debt.create",
            json!({"name": "Pré", "kind": "personal", "principal": 6_000_000, "annual_rate": "12.682503",
                   "system": "price", "contract_date": "2025-12-15", "first_due_date": "2026-01-15", "term_months": 48}),
        )
        .unwrap();
    let s = api.call("debt.schedule", json!({"id": pre["id"]})).unwrap();
    assert!(s[0]["index_factor"].is_null());
    assert_eq!(s[0]["due_date"], "2026-01-15");
}

#[test]
fn new_index_values_refresh_open_installments_once() {
    let (_d, api, _acc) = api();
    seed_daily(&api, "cdi", "2026-01-01", "2026-02-27", "0.05");
    let l = api
        .call(
            "debt.create",
            json!({"name": "CCB", "kind": "other", "principal": 1_000_000, "annual_rate": "1",
                   "indexer": "cdi", "system": "sac", "contract_date": "2026-01-02",
                   "first_due_date": "2026-02-02", "term_months": 4}),
        )
        .unwrap();
    let id = l["id"].as_str().unwrap().to_string();
    let before = api.call("debt.schedule", json!({"id": id})).unwrap();
    assert_eq!(before[1]["estimated"], false, "fevereiro já publicado");
    assert_eq!(before[2]["estimated"], true, "março sem CDI publicado");
    // Chega o CDI de março, mais alto que o último conhecido.
    seed_daily(&api, "cdi", "2026-03-02", "2026-04-03", "0.06");
    let n = api
        .with_session(|s, _| super::refresh_indexed(s.db.conn_mut()))
        .unwrap();
    assert_eq!(n, 1);
    let after = api.call("debt.schedule", json!({"id": id})).unwrap();
    assert_eq!(after[2]["estimated"], false);
    assert!(after[2]["interest"].as_i64().unwrap() > before[2]["interest"].as_i64().unwrap());
    // Sem índice novo, nada muda nem entra no histórico.
    let n = api
        .with_session(|s, _| super::refresh_indexed(s.db.conn_mut()))
        .unwrap();
    assert_eq!(n, 0);
}

/// Variações mensais (% a.m.) datadas no dia 1 de cada mês.
fn seed_monthly(api: &Api, series: &str, values: &[(&str, &str)]) {
    let vals: Vec<_> = values
        .iter()
        .map(|(d, v)| {
            (
                crate::time::parse_date(d).unwrap(),
                v.parse().unwrap(),
                false,
            )
        })
        .collect();
    api.with_session(|s, _| crate::indices::upsert_values(s.db.conn(), series, &vals, "teste"))
        .unwrap();
}

fn loan(api: &Api, extra: serde_json::Value) -> (String, Vec<serde_json::Value>) {
    let mut body = json!({"name": "Contrato", "kind": "mortgage", "principal": 10_000_000, "annual_rate": "0",
                          "system": "sac", "contract_date": "2026-01-10", "first_due_date": "2026-02-10",
                          "term_months": 4});
    for (k, v) in extra.as_object().unwrap() {
        body[k] = v.clone();
    }
    let l = api.call("debt.create", body).unwrap();
    let id = l["id"].as_str().unwrap().to_string();
    let s = api.call("debt.schedule", json!({"id": id})).unwrap();
    (id, s.as_array().unwrap().clone())
}

#[test]
fn monthly_correction_lag_floor_and_prorata() {
    let (_d, api, _acc) = api();
    seed_monthly(
        &api,
        "ipca",
        &[
            ("2025-11-01", "0.50"),
            ("2025-12-01", "1.00"),
            ("2026-01-01", "-0.40"),
            ("2026-02-01", "0.30"),
        ],
    );
    // Padrão (M − 1): a parcela de fevereiro corrige pelo IPCA de janeiro (−0,40%).
    let (_, a) = loan(&api, json!({"indexer": "ipca"}));
    assert_eq!(a[0]["adjustment"].as_i64().unwrap(), -40_000);
    // M − 2: dezembro (+1,00%).
    let (_, b) = loan(&api, json!({"indexer": "ipca", "correction_lag_months": 2}));
    assert_eq!(b[0]["adjustment"].as_i64().unwrap(), 100_000);
    // Piso zero: a deflação de janeiro não reduz o saldo.
    let (_, c) = loan(&api, json!({"indexer": "ipca", "deflation_floor": true}));
    assert_eq!(c[0]["adjustment"].as_i64().unwrap(), 0);
    // Pró-rata por dias corridos: contrato em 25/01, 1º vencimento 10/02 → 16 de 31 dias.
    let (_, d) = loan(
        &api,
        json!({"indexer": "ipca", "correction_lag_months": 2, "correction_prorata": "calendar",
               "contract_date": "2026-01-25"}),
    );
    let expected = (10_000_000f64 * (1.01f64.powf(16.0 / 31.0) - 1.0)).round() as i64;
    assert!(
        (d[0]["adjustment"].as_i64().unwrap() - expected).abs() <= 1,
        "{}",
        d[0]
    );
}

#[test]
fn tr_by_anniversary_and_interest_by_business_days() {
    use crate::finance::calendar::business_days_between;
    let (_d, api, _acc) = api();
    // TR de cada período que começa no dia (SGS 226): só os dias de aniversário importam.
    seed_daily(&api, "tr", "2026-01-01", "2026-06-30", "0.01");
    seed_monthly(
        &api,
        "tr",
        &[("2026-01-10", "0.20"), ("2026-02-10", "0.15")],
    );
    let (_, t) = loan(&api, json!({"indexer": "tr", "annual_rate": "9"}));
    // 1ª parcela: TR do período que começa na assinatura (10/01); 2ª, no 1º vencimento.
    assert_eq!(t[0]["adjustment"].as_i64().unwrap(), 20_000);
    let bal = 10_000_000 + 20_000 - t[0]["amortization"].as_i64().unwrap();
    assert_eq!(
        t[1]["adjustment"].as_i64().unwrap(),
        (bal as f64 * 0.0015).round() as i64
    );

    // Juros reais por dias úteis: (1 + 6%)^(du/252) no período.
    let (_, r) = loan(&api, json!({"annual_rate": "6", "interest_basis": "bd252"}));
    let du = business_days_between(
        crate::time::parse_date("2026-01-10").unwrap(),
        crate::time::parse_date("2026-02-10").unwrap(),
    );
    let expected = (10_000_000f64 * (1.06f64.powf(du as f64 / 252.0) - 1.0)).round() as i64;
    assert!(
        (r[0]["interest"].as_i64().unwrap() - expected).abs() <= 1,
        "{}",
        r[0]
    );
    assert_eq!(r[0]["business_days"].as_i64().unwrap(), du);
}

#[test]
fn past_installments_paid_on_create_and_in_bulk() {
    let (_d, api, _acc) = api();
    let today = crate::time::today_local();
    let contract = crate::time::add_months(today, -7);
    let first = crate::time::add_months(contract, 1);
    let body = |mark: bool| {
        json!({"name": "Antigo", "kind": "personal", "principal": 1_200_000, "annual_rate": "12",
               "system": "price", "contract_date": crate::time::fmt_date(contract),
               "first_due_date": crate::time::fmt_date(first), "term_months": 24, "mark_past_paid": mark})
    };
    let v = api.call("debt.create", body(true)).unwrap();
    assert_eq!(v["overdue_count"], 0);
    let paid = v["paid_count"].as_u64().unwrap();
    assert!((6..=7).contains(&paid), "{paid}");
    // Sem marcar, ficam atrasadas; o botão em lote resolve.
    let v = api.call("debt.create", body(false)).unwrap();
    assert!(v["overdue_count"].as_u64().unwrap() >= 6);
    let r = api
        .call(
            "debt.mark_paid_until",
            json!({"loan_id": v["id"], "before": crate::time::fmt_date(today)}),
        )
        .unwrap();
    assert_eq!(
        r["marked"].as_u64().unwrap(),
        v["overdue_count"].as_u64().unwrap()
    );
    let v = api.call("debt.get", json!({"id": v["id"]})).unwrap();
    assert_eq!(v["overdue_count"], 0);
}

#[test]
fn anticipation_discounts_interest_from_last_or_next() {
    let (_d, api, acc) = api();
    let l = api
        .call(
            "debt.create",
            json!({"name": "CDC", "kind": "personal", "principal": 6_000_000, "annual_rate": "12.682503",
                   "system": "price", "contract_date": "2026-09-15", "first_due_date": "2026-10-15",
                   "term_months": 48, "debit_account_id": acc}),
        )
        .unwrap();
    let id = l["id"].as_str().unwrap().to_string();
    let before = api.call("debt.get", json!({"id": id})).unwrap();
    // Das últimas: a 48ª (out/2030) a 1% a.m. vale bem menos hoje.
    let s = api
        .call(
            "debt.simulate_anticipation",
            json!({"loan_id": id, "date": "2026-10-20", "count": 2, "order": "last"}),
        )
        .unwrap();
    let nums: Vec<u64> = s["installments"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["number"].as_u64().unwrap())
        .collect();
    assert_eq!(nums, vec![47, 48]);
    let nominal = s["nominal_total"].as_i64().unwrap();
    let to_pay = s["to_pay"].as_i64().unwrap();
    assert!(to_pay < nominal * 65 / 100, "{s}");
    assert_eq!(s["discount"].as_i64().unwrap(), nominal - to_pay);

    // Das próximas: pouco desconto (vencem logo).
    let n = api
        .call(
            "debt.simulate_anticipation",
            json!({"loan_id": id, "date": "2026-10-20", "count": 2, "order": "next"}),
        )
        .unwrap();
    let nums: Vec<u64> = n["installments"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["number"].as_u64().unwrap())
        .collect();
    assert_eq!(nums, vec![2, 3]);
    assert!(n["discount"].as_i64().unwrap() < s["discount"].as_i64().unwrap());

    // Registrar: as duas últimas ficam pagas pelo valor presente, com o lançamento na conta,
    // os juros a pagar caem e o saldo devedor perde o principal delas.
    api.call(
        "debt.anticipate",
        json!({"loan_id": id, "date": "2026-10-20", "count": 2, "order": "last"}),
    )
    .unwrap();
    let after = api.call("debt.get", json!({"id": id})).unwrap();
    assert!(
        after["interest_remaining"].as_i64().unwrap()
            < before["interest_remaining"].as_i64().unwrap()
    );
    let sched = api.call("debt.schedule", json!({"id": id})).unwrap();
    let last = &sched.as_array().unwrap()[47];
    assert_eq!(last["paid"], true);
    assert_eq!(last["paid_date"], "2026-10-20");
    let amort_47_48: i64 = sched.as_array().unwrap()[46..]
        .iter()
        .map(|i| i["amortization"].as_i64().unwrap())
        .sum();
    let bal = api
        .call("reports.net_worth", json!({"as_of": "2026-10-21"}))
        .unwrap()["debts"]
        .as_i64()
        .unwrap();
    let first_after = sched.as_array().unwrap()[0]["balance_after"]
        .as_i64()
        .unwrap();
    assert_eq!(bal, first_after - amort_47_48);
    let txns = api
        .call("transactions.query", json!({"account_ids": [acc]}))
        .unwrap();
    assert!(txns.to_string().contains("Antecipação de 2 parcela(s)"));
}

#[test]
fn payments_in_the_linked_category_pay_the_nearest_installment() {
    let (_d, api, acc) = api();
    let cats = api.call("categories.list", json!({})).unwrap();
    let find = |name: &str| {
        cats.as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let cat = find("Juros e multas");
    let l = api
        .call(
            "debt.create",
            json!({"name": "Consignado", "kind": "payroll", "principal": 1_000_000, "annual_rate": "20",
                   "system": "price", "contract_date": "2026-01-10", "first_due_date": "2026-02-10",
                   "term_months": 12, "category_id": cat}),
        )
        .unwrap();
    let id = l["id"].as_str().unwrap().to_string();
    assert_eq!(l["category_id"], cat.as_str());
    // Receita não serve, e a mesma categoria não liga dois contratos.
    let income = find("Salário");
    assert!(
        api.call("debt.update", json!({"id": id, "category_id": income}))
            .is_err()
    );
    assert!(
        api.call(
            "debt.create",
            json!({"name": "Outro", "kind": "personal", "principal": 100_000, "annual_rate": "10",
                   "system": "price", "contract_date": "2026-01-10", "first_due_date": "2026-02-10",
                   "term_months": 6, "category_id": cat}),
        )
        .is_err()
    );
    // Lançamento na categoria, com valor diferente do cronograma (seguro, atraso): paga a
    // parcela de fevereiro.
    api.call(
        "transactions.create",
        json!({"account_id": acc, "date": "2026-02-12", "amount": -95_000,
               "description": "DEB CONSIGNADO", "category_id": cat}),
    )
    .unwrap();
    let sched = api.call("debt.schedule", json!({"id": id})).unwrap();
    let first = &sched.as_array().unwrap()[0];
    assert_eq!(first["paid"], true, "{first}");
    assert_eq!(first["paid_amount"], 95_000);
    assert_eq!(sched.as_array().unwrap()[1]["paid"], false);
    // Categorizar depois também liga (março).
    let t = api
        .call(
            "transactions.create",
            json!({"account_id": acc, "date": "2026-03-09", "amount": -93_000,
                   "description": "DEB CONSIGNADO 2"}),
        )
        .unwrap();
    api.call(
        "transactions.set_category",
        json!({"ids": [t["id"]], "category_id": cat}),
    )
    .unwrap();
    let sched = api.call("debt.schedule", json!({"id": id})).unwrap();
    assert_eq!(sched.as_array().unwrap()[1]["txn_id"], t["id"]);
}

#[test]
fn mortgage_insurance_mip_on_balance_and_dfi_on_property() {
    let (_d, api, _acc) = api();
    let l = api
        .call(
            "debt.create",
            json!({"name": "Apartamento", "kind": "mortgage", "principal": 40_000_000, "annual_rate": "12",
                   "system": "sac", "contract_date": "2026-01-05", "first_due_date": "2026-02-05",
                   "term_months": 360, "fees_monthly": 2_500, "property_value": 50_000_000,
                   "mip_rate": "0,0247", "dfi_rate": "0,0075"}),
        )
        .unwrap();
    let id = l["id"].as_str().unwrap().to_string();
    assert_eq!(l["mip_rate"], "0.0247");
    assert_eq!(l["property_value"], 50_000_000);
    let sched = api.call("debt.schedule", json!({"id": id})).unwrap();
    let rows = sched.as_array().unwrap();
    // MIP: 0,0247% de R$ 400.000 = R$ 98,80; DFI: 0,0075% de R$ 500.000 = R$ 37,50.
    assert_eq!(rows[0]["insurance_mip"], 9_880);
    assert_eq!(rows[0]["insurance_dfi"], 3_750);
    assert_eq!(rows[0]["insurance"], 9_880 + 3_750);
    assert_eq!(rows[0]["fees"], 2_500);
    let first = &rows[0];
    assert_eq!(
        first["total"].as_i64().unwrap(),
        first["amortization"].as_i64().unwrap()
            + first["interest"].as_i64().unwrap()
            + 9_880
            + 3_750
            + 2_500
    );
    // O MIP cai com o saldo; o DFI fica.
    let last = rows.last().unwrap();
    assert!(last["insurance_mip"].as_i64().unwrap() < 100);
    assert_eq!(last["insurance_dfi"], 3_750);
    // DFI sem valor do imóvel: recusado. Tirar o MIP recalcula as parcelas.
    assert!(
        api.call("debt.update", json!({"id": id, "property_value": null}))
            .is_err()
    );
    api.call("debt.update", json!({"id": id, "mip_rate": null}))
        .unwrap();
    let sched = api.call("debt.schedule", json!({"id": id})).unwrap();
    assert_eq!(sched[0]["insurance_mip"], 0);
    assert_eq!(sched[0]["insurance"], 3_750);
}

#[test]
fn every_term_can_be_edited_keeping_what_was_paid_and_amortized() {
    let (_d, api, acc) = api();
    let base = json!({"name": "Empréstimo", "kind": "personal", "principal": 1_000_000, "annual_rate": "24",
                      "system": "price", "contract_date": "2026-01-05", "first_due_date": "2026-02-05",
                      "term_months": 24, "debit_account_id": acc});
    let l = api.call("debt.create", base.clone()).unwrap();
    let id = l["id"].as_str().unwrap().to_string();
    let sched = api.call("debt.schedule", json!({"id": id})).unwrap();
    let first = sched[0]["id"].as_str().unwrap().to_string();
    // O usuário registrou o pagamento real (valor diferente do cronograma) e uma amortização.
    api.call(
        "debt.mark_paid",
        json!({"installment_id": first, "date": "2026-02-06", "amount": 55_555}),
    )
    .unwrap();
    api.call(
        "debt.add_extra_payment",
        json!({"loan_id": id, "date": "2026-03-20", "amount": 100_000, "mode": "reduce_term", "without_txn": true}),
    )
    .unwrap();

    // Correção do contrato: valor, taxa pela parcela, sistema, prazo, datas e indexador.
    let mut terms = base.clone();
    terms["name"] = json!("Empréstimo corrigido");
    terms["principal"] = json!(1_200_000);
    terms["annual_rate"] = json!("");
    terms["installment_amount"] = json!(70_000);
    terms["term_months"] = json!(30);
    terms["indexer"] = json!("ipca");
    let v = api
        .call("debt.update", json!({"id": id, "terms": terms}))
        .unwrap();
    assert_eq!(v["name"], "Empréstimo corrigido");
    assert_eq!(v["principal"], 1_200_000);
    assert_eq!(v["term_months"], 30);
    assert_eq!(v["indexer"], "ipca");
    assert_ne!(v["annual_rate"], "24", "taxa recalculada pela parcela");

    let after = api.call("debt.schedule", json!({"id": id})).unwrap();
    let rows = after.as_array().unwrap();
    let paid = rows.iter().find(|r| r["id"] == first.as_str()).unwrap();
    assert_eq!(paid["paid"], true);
    assert_eq!(
        paid["paid_amount"], 55_555,
        "o pagamento registrado à mão fica"
    );
    assert!(
        rows.iter()
            .any(|r| r["extra_payment"].as_i64().unwrap_or(0) == 100_000)
    );
    // Termos inválidos não mexem em nada.
    let mut bad = terms.clone();
    bad["term_months"] = json!(0);
    assert!(
        api.call("debt.update", json!({"id": id, "terms": bad}))
            .is_err()
    );
    assert_eq!(
        api.call("debt.get", json!({"id": id})).unwrap()["term_months"],
        30
    );
}

#[test]
fn housing_insurance_in_fixed_amounts_instead_of_rates() {
    let (_d, api, acc) = api();
    // Contrato com MIP e DFI em reais por mês (sem valor do imóvel).
    let l = api
        .call(
            "debt.create",
            json!({"name": "Apartamento", "kind": "mortgage", "principal": 30_000_000, "annual_rate": "9.5",
                   "system": "sac", "contract_date": "2026-01-10", "first_due_date": "2026-02-10",
                   "term_months": 120, "debit_account_id": acc, "mip_amount": 12_000, "dfi_amount": 3_000}),
        )
        .unwrap();
    assert_eq!(
        (l["mip_amount"].as_i64(), l["dfi_amount"].as_i64()),
        (Some(12_000), Some(3_000))
    );
    assert!(l["mip_rate"].is_null());
    let id = l["id"].as_str().unwrap().to_string();
    let sched = api.call("debt.schedule", json!({"id": id})).unwrap();
    for k in [0, 60, 119] {
        assert_eq!(sched[k]["insurance_mip"], 12_000, "MIP fixo na parcela {k}");
        assert_eq!(sched[k]["insurance_dfi"], 3_000);
    }
    // Voltar para percentual: o MIP passa a cair com o saldo.
    api.call(
        "debt.update",
        json!({"id": id, "mip_amount": null, "mip_rate": "0,03", "property_value": 40_000_000}),
    )
    .unwrap();
    let sched = api.call("debt.schedule", json!({"id": id})).unwrap();
    assert!(
        sched[0]["insurance_mip"].as_i64().unwrap() > sched[100]["insurance_mip"].as_i64().unwrap()
    );
    assert_eq!(sched[0]["insurance_dfi"], 3_000, "DFI fixo continua");
    assert!(
        api.call("debt.update", json!({"id": id, "mip_amount": -1}))
            .is_err()
    );
}

/// Conferência local com um contrato e um extrato reais, guardados fora do repositório:
/// `TREASURY_LOAN_CONTRACT=<json>` (os campos de `debt.create`, sem `debit_account_id`) e
/// `TREASURY_LOAN_STATEMENT=<tsv>`. Nada do contrato real fica no código.
#[test]
#[ignore]
fn real_statement_from_env() {
    let (Ok(contract), Ok(path)) = (
        std::env::var("TREASURY_LOAN_CONTRACT"),
        std::env::var("TREASURY_LOAN_STATEMENT"),
    ) else {
        return;
    };
    let (_d, api, acc) = api();
    let mut args: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(contract).unwrap()).unwrap();
    args["debit_account_id"] = json!(acc);
    let l = api.call("debt.create", args).unwrap();
    let id = l["id"].as_str().unwrap().to_string();
    let text = std::fs::read_to_string(path).unwrap();
    let p = api
        .call("debt.statement_preview", json!({"id": id, "text": text}))
        .unwrap();
    println!(
        "linhas {} de {} a {}",
        p["rows"].as_array().unwrap().len(),
        p["first_date"],
        p["last_date"]
    );
    println!("saldo {} prazo {}", p["last_balance"], p["remaining_term"]);
    println!(
        "taxa a.m. {} nominal {} efetiva {}",
        p["implied_monthly_pct"],
        p["implied_nominal_annual_pct"],
        p["implied_effective_annual_pct"]
    );
    println!("amortizações {}", p["extras"]);
    println!("avisos {}", p["warnings"]);
    let r = api
        .call("debt.import_statement", json!({"id": id, "text": text}))
        .unwrap();
    println!(
        "importadas {} anteriores pagas {}",
        r["imported"], r["marked_previous"]
    );
    let v = api.call("debt.get", json!({"id": id})).unwrap();
    println!(
        "saldo hoje {} pagas {} faltam {}",
        v["balance"], v["paid_count"], v["remaining_count"]
    );
    let s = api.call("debt.schedule", json!({"id": id})).unwrap();
    let versions: Vec<String> = s
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["version_number"] == 1)
        .map(|i| format!("v{} em {}", i["version"], i["due_date"]))
        .collect();
    println!("versões: {versions:?}");
    for i in s
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["paid"] == false)
        .take(3)
    {
        println!(
            "{} {}: amort {} juros {} correção {} total {} saldo {}",
            i["number"],
            i["due_date"],
            i["amortization"],
            i["interest"],
            i["adjustment"],
            i["total"],
            i["balance_after"]
        );
    }
    let last = s.as_array().unwrap().last().unwrap();
    println!(
        "última {} {} saldo {}",
        last["number"], last["due_date"], last["balance_after"]
    );
}

#[test]
fn contract_statement_replaces_history_and_projects_from_the_real_balance() {
    let (_d, api, acc) = api();
    let l = api
        .call(
            "debt.create",
            json!({"name": "Casa", "kind": "mortgage", "principal": 10_000_000, "annual_rate": "12",
                   "system": "price", "contract_date": "2025-12-10", "first_due_date": "2026-01-10",
                   "term_months": 120, "debit_account_id": acc, "fees_monthly": 2_500}),
        )
        .unwrap();
    let id = l["id"].as_str().unwrap().to_string();
    // Extrato (fictício): na 3ª parcela o saldo cai 20 mil a mais (amortização extraordinária).
    let text = "Data\tPrestação\tAmortização\tJuros\tSeguros\tTaxas\tSaldo devedor\tCorreção monetária\tValor pago\n\
                10/01/2026\t1.461,40\t484,80\t951,60\t0,00\t25,00\t99.515,20\t0,00\t1.461,40\n\
                10/02/2026\t1.461,40\t489,41\t946,99\t0,00\t25,00\t99.025,79\t0,00\t1.461,40\n\
                10/03/2026\t1.461,40\t494,07\t942,33\t0,00\t25,00\t78.531,72\t0,00\t1.500,00";
    let p = api
        .call(
            "debt.statement_preview",
            json!({"id": id, "text": text, "remaining_term": 60}),
        )
        .unwrap();
    assert_eq!(p["rows"].as_array().unwrap().len(), 3);
    assert_eq!(p["extras"][0]["date"], "2026-03-10");
    assert_eq!(p["extras"][0]["amount"], 2_000_000);
    assert_eq!(p["remaining_term"], 60);
    let r = api
        .call(
            "debt.import_statement",
            json!({"id": id, "text": text, "remaining_term": 60}),
        )
        .unwrap();
    assert_eq!(r["imported"], 3);
    let v = api.call("debt.get", json!({"id": id})).unwrap();
    assert_eq!(v["statement_until"], 3);
    assert_eq!(v["remaining_count"], 60);
    let s = api.call("debt.schedule", json!({"id": id})).unwrap();
    let rows = s.as_array().unwrap();
    let third = rows.iter().find(|i| i["number"] == 3).unwrap();
    assert_eq!(third["paid"], true);
    assert_eq!(third["paid_amount"], 150_000, "valor pago do extrato");
    assert_eq!(third["extra_payment"], 2_000_000);
    // A projeção parte do saldo real: a 4ª parcela amortiza sobre 78.531,72 em 60 meses.
    let fourth = rows.iter().find(|i| i["number"] == 4).unwrap();
    assert_eq!(fourth["due_date"], "2026-04-10");
    let bal = fourth["balance_after"].as_i64().unwrap();
    assert!((7_700_000..7_850_000).contains(&bal), "{bal}");
    assert_eq!(rows.last().unwrap()["balance_after"], 0);
    // Editar o contrato depois mantém o extrato.
    api.call("debt.update", json!({"id": id, "annual_rate": "11.5"}))
        .unwrap();
    let v = api.call("debt.get", json!({"id": id})).unwrap();
    assert_eq!(
        (v["statement_until"].as_i64(), v["remaining_count"].as_i64()),
        (Some(3), Some(60))
    );
}

#[test]
fn payment_with_balance_syncs_and_amortization_opens_a_new_version() {
    let (_d, api, acc) = api();
    let l = api
        .call(
            "debt.create",
            json!({"name": "Casa", "kind": "mortgage", "principal": 10_000_000, "annual_rate": "12",
                   "system": "price", "contract_date": "2025-12-10", "first_due_date": "2026-01-10",
                   "term_months": 120, "debit_account_id": acc}),
        )
        .unwrap();
    let id = l["id"].as_str().unwrap().to_string();
    let s = api.call("debt.schedule", json!({"id": id})).unwrap();
    // Parcela 1 como no print: valor pago e saldo depois.
    let p1 = api
        .call(
            "debt.record_payment",
            json!({"id": id, "date": "2026-01-10", "amount": 143_700, "balance_after": s[0]["balance_after"]}),
        )
        .unwrap();
    assert_eq!(
        (p1["paid"].as_bool(), p1["paid_amount"].as_i64()),
        (Some(true), Some(143_700))
    );
    // Empréstimo ou parcela que não existem: o erro diz qual.
    let e = api
        .call(
            "debt.record_payment",
            json!({"id": "nao-existe", "date": "2026-01-10", "amount": 1}),
        )
        .unwrap_err()
        .to_string();
    assert!(e.contains("empréstimo nao-existe"), "{e}");
    let e = api
        .call(
            "debt.record_payment",
            json!({"id": id, "installment_id": "x", "date": "2026-01-10", "amount": 1}),
        )
        .unwrap_err()
        .to_string();
    assert!(e.contains("parcela x"), "{e}");
    // Parcela 2: o banco mostra a amortização da parcela e o saldo 20 mil menor (amortização
    // extraordinária reduzindo o prazo para 60).
    let amort = s[1]["amortization"].as_i64().unwrap();
    let bal = s[1]["balance_after"].as_i64().unwrap() - 2_000_000;
    api.call(
        "debt.record_payment",
        json!({"id": id, "date": "2026-02-10", "amortization": amort, "interest": s[1]["interest"],
               "balance_after": bal, "remaining_term": 60}),
    )
    .unwrap();
    let v = api.call("debt.get", json!({"id": id})).unwrap();
    assert_eq!(v["remaining_count"], 60);
    let s = api.call("debt.schedule", json!({"id": id})).unwrap();
    let rows = s.as_array().unwrap();
    assert_eq!(rows[1]["balance_after"], bal, "saldo do print");
    assert_eq!(rows[1]["extra_payment"], 2_000_000);
    // Versões: a 2ª parcela abre a versão 2 e a numeração recomeça.
    assert_eq!(
        (
            rows[0]["version"].as_i64(),
            rows[0]["version_number"].as_i64()
        ),
        (Some(1), Some(1))
    );
    assert_eq!(
        (
            rows[1]["version"].as_i64(),
            rows[1]["version_number"].as_i64()
        ),
        (Some(2), Some(1))
    );
    assert_eq!(
        (
            rows[2]["version"].as_i64(),
            rows[2]["version_number"].as_i64()
        ),
        (Some(2), Some(2))
    );
}

#[test]
fn payoff_plan_uses_registered_loans() {
    let (_d, api, acc) = api();
    api.call(
        "debt.create",
        json!({"name": "Carro", "kind": "vehicle", "principal": 6_000_000, "annual_rate": "12.682503",
               "system": "price", "contract_date": "2025-12-15", "first_due_date": "2026-01-15", "term_months": 48,
               "debit_account_id": acc}),
    )
    .unwrap();
    let p = api
        .call(
            "debt.payoff_plan",
            json!({"extra_monthly": 100_000, "custom": [{"name": "Cheque especial", "balance": 200_000, "annual_rate": "140", "min_payment": 20_000}]}),
        )
        .unwrap();
    assert_eq!(p["debts"].as_array().unwrap().len(), 2);
    let car = &p["debts"][0];
    assert_eq!(car["name"], "Carro");
    assert!((car["min_payment"].as_i64().unwrap() - 158_002).abs() <= 2);
    assert_eq!(p["avalanche"]["debts"][0]["name"], "Cheque especial");
    assert!(p["avalanche"]["interest_saved"].as_i64().unwrap() > 0);
    assert!(p["avalanche"]["months"].as_u64() < p["minimum"]["months"].as_u64());
}

/// Plano de quitação, erros e o motivo no histórico saem no idioma da instalação.
#[test]
fn texts_follow_the_installation_locale() {
    use crate::i18n::{Locale, set_current};
    let (_d, api, acc) = api();
    set_current(Locale::En);
    api.call(
        "debt.create",
        json!({"name": "Carro", "kind": "vehicle", "principal": 6_000_000, "annual_rate": "12.682503",
               "system": "price", "contract_date": "2025-12-15", "first_due_date": "2026-01-15", "term_months": 48,
               "debit_account_id": acc}),
    )
    .unwrap();
    let p = api
        .call(
            "debt.payoff_plan",
            json!({"extra_monthly": 100_000, "custom": [{"name": "Cheque especial", "balance": 200_000, "annual_rate": "140", "min_payment": 20_000}]}),
        )
        .unwrap();
    let err = api
        .call(
            "debt.create",
            json!({"name": "X", "kind": "personal", "principal": 100_000, "annual_rate": "20", "system": "price",
                   "contract_date": "2026-02-01", "first_due_date": "2026-01-01", "term_months": 12}),
        )
        .unwrap_err()
        .message();
    let history = api.call("changes.list", json!({"limit": 5})).unwrap();
    set_current(Locale::PtBr);
    let explanation = p["explanation"].as_str().unwrap();
    assert!(explanation.contains("Snowball: "), "{explanation}");
    assert!(
        explanation.starts_with("Paying only the minimum: debt-free in "),
        "{explanation}"
    );
    assert_eq!(
        err,
        "the first installment can't be due before the loan date"
    );
    assert!(
        history.to_string().contains("Loan added: Carro"),
        "{history}"
    );
}

/// "4,75 + TR" no campo da taxa (pedido de 21/10/2026): o contrato fica com spread de 4,75%
/// a.a. e indexador TR, como se os dois campos tivessem sido preenchidos. Indexador diferente
/// do escolhido é recusado. Também o limite da conta (cheque especial) aceita a taxa assim e
/// estima os juros com o último valor da TR do cofre, mostrando a data dele.
#[test]
fn rate_plus_tr_in_one_field_for_loans_and_the_credit_limit() {
    let (_d, api, acc) = api();
    let l = api
        .call(
            "debt.create",
            json!({"name": "Imóvel Caixa", "kind": "mortgage", "principal": 30_000_000, "annual_rate": "4,75 + TR",
                   "system": "sac", "contract_date": "2026-01-01", "first_due_date": "2026-02-01", "term_months": 360,
                   "debit_account_id": acc}),
        )
        .unwrap();
    let v = api.call("debt.get", json!({"id": l["id"]})).unwrap();
    assert_eq!(v["indexer"], "tr", "{v}");
    assert_eq!(v["annual_rate"], "4.75");
    assert!(
        api.call(
            "debt.create",
            json!({"name": "X", "kind": "mortgage", "principal": 1_000_000, "annual_rate": "4,75 + TR", "indexer": "ipca",
                   "system": "sac", "contract_date": "2026-01-01", "first_due_date": "2026-02-01", "term_months": 12}),
        )
        .is_err()
    );

    // Limite de crédito com "4,75 + TR" e a TR de outubro no cofre.
    api.with_session(|s, _| {
        Ok(s.db.conn().execute(
            "INSERT INTO index_value (series_id, date, value, source, fetched_at)
             VALUES ('tr', ?1, '0.15', 'test', 'x')",
            [crate::time::fmt_date(crate::time::add_months(
                crate::time::today_local(),
                -1,
            ))],
        )?)
    })
    .unwrap();
    api.call(
        "accounts.update",
        json!({"id": acc, "overdraft": {"limit": 1_000_000, "annual_rate": "4,75 + TR"}}),
    )
    .unwrap();
    api.call(
        "transactions.create",
        json!({"account_id": acc, "date": crate::time::fmt_date(crate::time::today_local()), "amount": -100_000, "description": "Saque"}),
    )
    .unwrap();
    let list = api.call("accounts.list", json!({})).unwrap();
    let a = list
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["id"] == acc.as_str())
        .unwrap();
    assert_eq!(a["overdraft"]["indexer"], "tr", "{a}");
    assert_eq!(a["overdraft"]["annual_rate"], "4.75");
    let st = &a["overdraft_status"];
    assert_eq!(st["index"]["monthly_pct"], "0.15", "{st}");
    assert!(st["index"]["date"].is_string());
    assert_eq!(st["estimated_interest"], 538, "R$ 1.000 a ≈ 0,538% a.m.");
}

/// Pelo MCP o extrato vira proposta: a resposta traz o resultado simulado e o resumo diz quantas
/// parcelas e o saldo, em vez de zero (feedback 01a1a3ad).
#[test]
fn statement_as_proposal_reports_the_simulated_result() {
    let (_d, api, acc) = api();
    let id = api
        .call(
            "debt.create",
            json!({"name": "Casa", "kind": "mortgage", "principal": 10_000_000, "annual_rate": "12",
                   "system": "price", "contract_date": "2025-12-10", "first_due_date": "2026-01-10",
                   "term_months": 120, "debit_account_id": acc, "fees_monthly": 2_500}),
        )
        .unwrap()["id"]
        .clone();
    let text = "Data\tPrestação\tAmortização\tJuros\tSeguros\tTaxas\tSaldo devedor\tCorreção monetária\tValor pago\n\
                10/01/2026\t1.461,40\t484,80\t951,60\t0,00\t25,00\t99.515,20\t0,00\t1.461,40\n\
                10/02/2026\t1.461,40\t489,41\t946,99\t0,00\t25,00\t99.025,79\t0,00\t1.461,40";
    let mcp = crate::api::Actor::Mcp {
        client: "teste".into(),
        read_only: false,
    };
    let r = api
        .call_as(
            &mcp,
            "debt.import_statement",
            json!({"id": id, "text": text, "remaining_term": 60}),
        )
        .unwrap();
    assert_eq!(r["status"], "pending_approval", "{r}");
    assert_eq!(r["report"]["imported"], 2, "{r}");
    let spec = crate::mcp::tools::TOOLS
        .iter()
        .find(|t| t.name == "import_loan_statement")
        .unwrap();
    let s = spec.summarize_in(&r, Some("BRL"));
    assert!(s.contains("2 parcela"), "{s}");
    assert!(s.contains("Proposta"), "{s}");
    assert!(s.contains("99.025,79"), "{s}");
}
