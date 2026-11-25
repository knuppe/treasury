//! Custody snapshots and subscriptions, fixed income by contract, Tesouro interest, bonus by
//! quantity, duplicates against brokerage notes and statements, and the custodian → account
//! choice (grouping, saved mapping, accounts created by the import), with made-up reports.

use base64::Engine as _;
use serde_json::{Value, json};

use crate::api::Api;
use crate::paths::TreasuryPaths;

const SUBSCRIPTION: &[u8] = include_bytes!("fixtures/subscricao-atualizacao.csv");

const MOV_HEADER: &str = "Entrada/Saída;Data;Movimentação;Produto;Instituição;Quantidade;Preço unitário;Valor da Operação\n";

fn file(name: &str, bytes: &[u8]) -> Value {
    json!({"filename": name, "content_base64": base64::engine::general_purpose::STANDARD.encode(bytes)})
}

fn mov(rows: &str) -> Vec<u8> {
    format!("{MOV_HEADER}{rows}").into_bytes()
}

struct Env {
    _dir: tempfile::TempDir,
    api: Api,
}

impl Env {
    fn new() -> Env {
        let dir = tempfile::tempdir().unwrap();
        let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
        let api = Api::new_unlocked_process(paths).unwrap();
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "Família", "members": ["Ana"]}),
        )
        .unwrap();
        Env { _dir: dir, api }
    }

    fn account(&self, name: &str, kind: &str) -> String {
        self.api
            .call(
                "accounts.create",
                json!({"name": name, "kind": kind, "opening_date": "2023-12-01"}),
            )
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn preview(&self, args: Value) -> Value {
        self.api.call("b3.preview", args).unwrap()
    }

    fn count(&self, sql: &str) -> i64 {
        self.api
            .with_session(|s, _| Ok(s.db.conn().query_row(sql, [], |r| r.get::<_, i64>(0))?))
            .unwrap()
    }
}

fn rows<'a>(v: &'a Value, list: &str) -> &'a Vec<Value> {
    v[list].as_array().unwrap()
}

fn line<'a>(p: &'a Value, key: &str) -> &'a Value {
    p["position_check"]["lines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["asset_key"] == key)
        .unwrap_or_else(|| panic!("no line {key}: {:#}", p["position_check"]))
}

// ---------------------------------------------------------------------------
// 2.1 "Atualização"

#[test]
fn update_after_exercised_subscription_is_a_buy_at_the_receipt_price() {
    let env = Env::new();
    env.account("XP Investimentos", "brokerage");
    let pos = "Produto;Instituição;Conta;Código de Negociação;Quantidade;Preço de Fechamento;Valor Atualizado\n\
               ABCD11 - FII FICTICIO ABCD;XP INVESTIMENTOS CCTVM S/A;123;ABCD11;10;101,00;1.010,00\n";
    let p = env.preview(json!({"files": [
        file("subscricao-atualizacao.csv", SUBSCRIPTION),
        file("posicao-fundos-2026-04-30.csv", pos.as_bytes()),
    ]}));
    let trades = rows(&p, "trades");
    assert_eq!(trades.len(), 1, "{trades:#?}");
    let t = &trades[0];
    assert_eq!(t["asset_key"], "ABCD11");
    assert_eq!(t["origin"], "subscription");
    assert_eq!(t["quantity"], "10");
    assert_eq!(t["price"], "100");
    assert_eq!(t["status"], "new");
    // The right and the receipt stay ignored.
    assert!(
        rows(&p, "ignored")
            .iter()
            .all(|i| i["rule"] == "subscription_right")
    );
    assert_eq!(line(&p, "ABCD11")["status"], "match");
}

#[test]
fn update_repeating_the_held_quantity_is_a_snapshot() {
    let env = Env::new();
    env.account("XP Investimentos", "brokerage");
    let m = mov(
        "Credito;15/03/2026;Dividendo;VALE3 - VALE S.A.;XP INVESTIMENTOS CCTVM S/A;111;0,50;55,50\n\
                 Credito;01/04/2026;Atualização;VALE3 - VALE S.A.;XP INVESTIMENTOS CCTVM S/A;111;-;-\n\
                 Credito;01/07/2026;Atualização;VALE3 - VALE S.A.;XP INVESTIMENTOS CCTVM S/A;111;-;-\n\
                 Credito;01/08/2026;Atualização;ABCD3 - EMPRESA FICTICIA;XP INVESTIMENTOS CCTVM S/A;7;-;-\n",
    );
    let p = env.preview(json!({"files": [file("mov.csv", &m)]}));
    let snapshots: Vec<_> = rows(&p, "ignored")
        .iter()
        .filter(|i| i["rule"] == "custody_snapshot")
        .collect();
    assert_eq!(snapshots.len(), 2);
    // Neither a subscription nor a known position: confirm, with the cost.
    let t = &rows(&p, "trades")[0];
    assert_eq!(t["asset_key"], "ABCD3");
    assert_eq!(t["origin"], "custody_update");
    assert_eq!(t["status"], "review");
    // Approving it needs the cost.
    let r = env
        .api
        .call(
            "b3.commit",
            json!({"token": p["token"], "approve_actions": [{"index": t["index"], "unit_cost": "12.5"}]}),
        )
        .unwrap();
    assert_eq!(r["trades"], 1, "{r:#}");
    assert_eq!(
        env.count("SELECT count(*) FROM trade WHERE price = '12.5' AND quantity = '7'"),
        1
    );
}

// ---------------------------------------------------------------------------
// 2.5 Fixed income by contract

#[test]
fn fixed_income_contract_is_one_asset_from_investment_to_redemption_and_position() {
    let env = Env::new();
    env.account("Viacredi", "checking");
    let inst = "COOPERATIVA DE CREDITO VALE DO ITAJAI - VIACREDI";
    let m = mov(&format!(
        "Credito;10/03/2026;APLICAÇÃO;RDB - RDB31497YWB;{inst};100000;0,01;1.000,00\n\
         Credito;12/03/2026;APLICAÇÃO;RDB - RDB00786TZO;{inst};60690;0,01;606,90\n\
         Debito;20/04/2026;RESGATE ANTECIPADO;RDB - RDB31497YWB - {inst};{inst};100000;0,0101;1.010,00\n"
    ));
    let pos = format!(
        "Produto;Instituição;Emissor;Código;Indexador;Tipo de regime;Data de Emissão;Vencimento;Quantidade;Quantidade Disponível;Quantidade Indisponível;Motivo;Contraparte;Preço Atualizado MTM;Valor Atualizado MTM;Preço Atualizado CURVA;Valor Atualizado CURVA;Preço Atualizado FECHAMENTO;Valor Atualizado FECHAMENTO\n\
         RDB - {inst};{inst};{inst};RDB00786TZO;DI;Depositada;12/03/2026;12/03/2028;60690;60690;-;-;-;-;-;0,0102;619,03;-;-\n"
    );
    let p = env.preview(json!({"files": [
        file("mov.csv", &m),
        file("posicao-renda-fixa-2026-04-30.csv", pos.as_bytes()),
    ]}));
    assert!(
        rows(&p, "ignored")
            .iter()
            .all(|i| i["rule"] != "unknown_movement")
    );
    let trades = rows(&p, "trades");
    assert_eq!(trades.len(), 3);
    assert!(
        trades.iter().all(|t| t["status"] == "new"),
        "{:#?} {:#?}",
        p["institutions"],
        trades[2]
    );
    let keys: Vec<&str> = trades
        .iter()
        .map(|t| t["asset_key"].as_str().unwrap())
        .collect();
    assert_eq!(keys.iter().filter(|k| **k == "rf:RDB31497YWB").count(), 2);
    let assets = rows(&p, "assets");
    assert_eq!(assets.len(), 2);
    let a = assets
        .iter()
        .find(|a| a["key"] == "rf:RDB00786TZO")
        .unwrap();
    assert_eq!(
        a["name"],
        format!("RDB {inst} RDB00786TZO"),
        "issuer from the position"
    );
    assert_eq!(a["fixed_income"]["issue_date"], "2026-03-12");
    assert_eq!(a["fixed_income"]["maturity_date"], "2028-03-12");
    // The CURVA price and value, not the empty MTM.
    let l = line(&p, "rf:RDB00786TZO");
    assert_eq!(l["status"], "match");
    assert_eq!(l["b3_value"], 61903);
    assert_eq!(l["close_price"], "0.0102");
}

// ---------------------------------------------------------------------------
// 2.3 Tesouro interest

#[test]
fn tesouro_interest_is_gross_with_regressive_tax_and_matches_by_isin() {
    let env = Env::new();
    env.account("XP Investimentos", "brokerage");
    let m = mov(
        "Credito;10/01/2025;Compra;Tesouro IPCA+ com Juros Semestrais 2035;XP INVESTIMENTOS CCTVM S/A;0,51;4.000,00;2.040,00\n\
         Credito;15/05/2025;Juros;Tesouro IPCA+ com Juros Semestrais 2035;XP INVESTIMENTOS CCTVM S/A;0,51;153,02;78,04\n\
         Credito;17/11/2025;Juros;Tesouro IPCA+ com Juros Semestrais 2035;XP INVESTIMENTOS CCTVM S/A;0,51;155,24;79,17\n",
    );
    let pos = "Produto;Instituição;Código ISIN;Indexador;Vencimento;Quantidade;Quantidade Disponível;Quantidade Indisponível;Motivo;Valor Aplicado;Valor bruto;Valor líquido;Valor Atualizado\n\
               TESOURO IPCA+ C/ JUROS SEMESTRAIS 2035;XP INVESTIMENTOS CCTVM S/A;BRSTNCNTB4U6;IPCA;15/05/2035;0,51;0,51;-;-;2.040,00;2.200,00;2.150,00;2.200,00\n";
    let p = env.preview(json!({"files": [
        file("mov.csv", &m),
        file("posicao-tesouro-direto-2025-11-30.csv", pos.as_bytes()),
    ]}));
    let incomes = rows(&p, "incomes");
    assert_eq!(incomes.len(), 2);
    // 125 days held → 22.5%; 311 days → 20%.
    let i0 = &incomes[0];
    assert_eq!(i0["gross"], 7804);
    assert_eq!(i0["tax_withheld"], 1756);
    assert_eq!(i0["net"], 7804 - 1756);
    let i1 = &incomes[1];
    assert_eq!(i1["gross"], 7917);
    assert_eq!(i1["tax_withheld"], 1583);
    // One asset for the movements and the position, with the position's ISIN.
    assert_eq!(rows(&p, "assets").len(), 1);
    assert_eq!(rows(&p, "assets")[0]["key"], "tesouro:ipca-js:2035");
    assert_eq!(line(&p, "tesouro:ipca-js:2035")["status"], "match");
    env.api
        .call("b3.commit", json!({"token": p["token"]}))
        .unwrap();
    assert_eq!(
        env.count("SELECT count(*) FROM asset WHERE isin = 'BRSTNCNTB4U6'"),
        1
    );
    // Without the purchase the tax is presumed at 15%.
    let env = Env::new();
    env.account("XP Investimentos", "brokerage");
    let m = mov(
        "Credito;17/11/2025;Juros;Tesouro IPCA+ com Juros Semestrais 2035;XP INVESTIMENTOS CCTVM S/A;0,51;155,24;79,17\n",
    );
    let p = env.preview(json!({"files": [file("mov.csv", &m)]}));
    let i = &rows(&p, "incomes")[0];
    assert_eq!(i["tax_withheld"], 1188);
    assert_eq!(i["status"], "new");
}

// ---------------------------------------------------------------------------
// 2.2 Bonus

#[test]
fn bonus_without_a_prior_position_credits_the_quantity() {
    let env = Env::new();
    let acc = env.account("XP Investimentos", "brokerage");
    let m = mov(
        "Credito;01/12/2025;Bonificação em Ativos;ITSA4 - ITAUSA S.A.;XP INVESTIMENTOS CCTVM S/A;30;-;-\n",
    );
    let p = env.preview(json!({"files": [file("mov.csv", &m)]}));
    let a = &rows(&p, "actions")[0];
    assert_eq!(a["kind"], "bonus");
    assert_eq!(a["status"], "new");
    assert_eq!(a["file_quantity"], "30");
    let r = env
        .api
        .call(
            "b3.commit",
            json!({"token": p["token"], "approve_actions": [{"index": a["index"], "unit_cost": "18.5"}]}),
        )
        .unwrap();
    assert_eq!(r["actions"], 1);
    let s = env
        .api
        .call("portfolio.summary", json!({"as_of": "2025-12-31"}))
        .unwrap();
    let pos = s["positions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["asset"]["ticker"] == "ITSA4")
        .unwrap();
    assert_eq!(pos["quantity"], "30");
    assert_eq!(pos["cost"], 55500);
    assert_eq!(
        env.count(&format!(
            "SELECT count(*) FROM corporate_action WHERE account_id = '{acc}' AND quantity = '30' AND factor IS NULL"
        )),
        1
    );
}

#[test]
fn bonus_with_a_prior_position_derives_the_factor_and_a_bought_bonus_is_flagged() {
    let env = Env::new();
    let acc = env.account("XP Investimentos", "brokerage");
    let id = env
        .api
        .call(
            "assets.create",
            json!({"ticker": "ITSA4", "name": "ITAUSA", "class": "stock_br"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let trade = |date: &str, qty: &str, price: &str| {
        env.api
            .call(
                "portfolio.create_trade",
                json!({"asset_id": id, "account_id": acc, "date": date, "side": "buy",
                       "quantity": qty, "price": price, "cash_movement": false}),
            )
            .unwrap();
    };
    trade("2025-11-02", "1500", "10");
    let m = mov(
        "Credito;01/12/2025;Bonificação em Ativos;ITSA4 - ITAUSA S.A.;XP INVESTIMENTOS CCTVM S/A;30;-;-\n",
    );
    let p = env.preview(json!({"files": [file("mov.csv", &m)]}));
    let a = &rows(&p, "actions")[0];
    assert_eq!(a["factor"], "0.02", "30 new shares for 1500");
    assert_eq!(a["status"], "new");
    // The same 30 shares recorded before as a buy, without a brokerage note.
    trade("2025-12-02", "30", "0");
    let p = env.preview(json!({"files": [file("mov.csv", &m)]}));
    assert_eq!(rows(&p, "actions")[0]["status"], "possible_duplicate");
}

// ---------------------------------------------------------------------------
// 2.4 Duplicates against brokerage notes and statements

#[test]
fn settlement_of_a_trade_from_a_brokerage_note_is_a_duplicate() {
    let env = Env::new();
    let acc = env.account("XP Investimentos", "brokerage");
    let id = env
        .api
        .call(
            "assets.create",
            json!({"ticker": "ITSA4", "name": "ITAUSA", "class": "stock_br"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    // The note's trade: two business days before the settlement on 08/10/2026.
    env.api
        .call(
            "portfolio.create_trade",
            json!({"asset_id": id, "account_id": acc, "date": "2026-10-06", "side": "buy",
                   "quantity": "125", "price": "10.02", "note_number": "123456", "cash_movement": false}),
        )
        .unwrap();
    let m = mov(
        "Credito;08/10/2026;Transferência - Liquidação;ITSA4 - ITAUSA S.A.;XP INVESTIMENTOS CCTVM S/A;125;10,00;1.250,00\n",
    );
    let p = env.preview(json!({"files": [file("mov.csv", &m)]}));
    let t = &rows(&p, "trades")[0];
    assert_eq!(t["status"], "duplicate");
    assert_eq!(t["date"], "2026-10-06");
}

#[test]
fn cash_movement_links_to_the_statement_entry_already_imported() {
    let env = Env::new();
    let acc = env.account("XP Investimentos", "brokerage");
    // The broker's statement (OFX) already has the dividend.
    env.api
        .call(
            "transactions.create",
            json!({"account_id": acc, "date": "2026-03-16", "amount": 5550, "description": "DIVIDENDOS VALE3"}),
        )
        .unwrap();
    let m = mov(
        "Credito;15/03/2026;Dividendo;VALE3 - VALE S.A.;XP INVESTIMENTOS CCTVM S/A;111;0,50;55,50\n",
    );
    let p = env.preview(json!({"files": [file("mov.csv", &m)]}));
    let before = env.count("SELECT count(*) FROM txn");
    env.api
        .call(
            "b3.commit",
            json!({"token": p["token"], "cash_movement": true}),
        )
        .unwrap();
    assert_eq!(env.count("SELECT count(*) FROM txn"), before);
    assert_eq!(
        env.count("SELECT count(*) FROM income_event WHERE txn_id IS NOT NULL"),
        1
    );
}

// ---------------------------------------------------------------------------
// 2.6.5 Opening positions

#[test]
fn opening_positions_come_from_the_evidence_and_fix_the_sales() {
    let env = Env::new();
    let acc = env.account("XP Investimentos", "brokerage");
    let m = mov(
        "Credito;15/03/2026;Dividendo;VALE3 - VALE S.A.;XP INVESTIMENTOS CCTVM S/A;111;0,50;55,50\n\
                 Debito;10/06/2026;Transferência - Liquidação;VALE3 - VALE S.A.;XP INVESTIMENTOS CCTVM S/A;111;60,00;6.660,00\n",
    );
    let p = env.preview(json!({"files": [file("mov.csv", &m)]}));
    // Without the opening position the sale is still imported, with a pending item.
    assert_eq!(rows(&p, "trades")[0]["status"], "warning");
    assert_eq!(p["counts"]["warnings"], 1);
    let o = &rows(&p, "openings")[0];
    assert_eq!(o["asset_key"], "VALE3");
    assert_eq!(o["quantity"], "111");
    assert_eq!(o["date"], "2026-03-14");
    assert_eq!(o["account_id"], acc.as_str());
    let p = env.preview(json!({"files": [file("mov.csv", &m)], "openings": [
        {"asset_key": "VALE3", "account_id": acc, "quantity": "111", "unit_cost": "50"}
    ]}));
    let trades = rows(&p, "trades");
    assert!(trades.iter().all(|t| t["status"] == "new"), "{trades:#?}");
    assert!(trades.iter().any(|t| t["origin"] == "opening"));
    assert_eq!(rows(&p, "openings")[0]["chosen"], true);
    assert_eq!(p["counts"]["errors"], 0);
    assert_eq!(p["counts"]["warnings"], 0);
}

// ---------------------------------------------------------------------------
// Sales whose purchase is older than the file

/// The sale is the official record: it is imported by default with a pending item, stays
/// out of the result until the purchase is known, and the person decides afterwards.
#[test]
fn sale_without_history_is_imported_and_waits_for_the_purchase() {
    let env = Env::new();
    let acc = env.account("XP Investimentos", "brokerage");
    let m = mov(
        "Debito;10/06/2026;Transferência - Liquidação;VALE3 - VALE S.A.;XP INVESTIMENTOS CCTVM S/A;100;60,00;6.000,00\n\
         Debito;10/07/2026;Transferência - Liquidação;PETR4 - PETROBRAS;XP INVESTIMENTOS CCTVM S/A;50;40,00;2.000,00\n",
    );
    let p = env.preview(json!({"files": [file("mov.csv", &m)]}));
    let trades = rows(&p, "trades");
    assert!(
        trades.iter().all(|t| t["status"] == "warning"),
        "{trades:#?}"
    );
    assert!(
        trades[0]["warning"]
            .as_str()
            .unwrap()
            .contains("maior que a posição de 0")
    );
    assert_eq!(p["counts"]["trades"], 2);
    assert_eq!(p["counts"]["warnings"], 2);
    assert_eq!(p["counts"]["errors"], 0);

    // Default: both are written. The person may still leave a row out (`skip`).
    let r = env
        .api
        .call("b3.commit", json!({"token": p["token"]}))
        .unwrap();
    assert_eq!(r["trades"], 2);
    assert_eq!(r["pending_sales"], 2);
    assert_eq!(r["errors"], json!([]));

    let pending = env
        .api
        .call("portfolio.uncovered_sales", json!({}))
        .unwrap();
    assert_eq!(pending["open"], 2);
    let rows_ = rows(&pending, "rows");
    // Most recent first.
    assert_eq!(rows_[0]["ticker"], "PETR4");
    assert_eq!(rows_[0]["missing_quantity"], "50");
    assert_eq!(rows_[0]["missing_value"], 200_000);
    assert_eq!(rows_[1]["ticker"], "VALE3");
    assert_eq!(rows_[1]["account_id"], acc.as_str());
    assert_eq!(rows_[1]["accepted"], false);

    // Nothing was made up: no position, no realized result, and "To solve" asks.
    let pf = env
        .api
        .call("portfolio.summary", json!({"include_closed": true}))
        .unwrap();
    assert_eq!(pf["realized"], 0, "{pf:#}");
    assert_eq!(pf["cost"], 0, "{pf:#}");
    let sales = env.api.call("portfolio.sales", json!({})).unwrap();
    assert_eq!(rows(&sales, "rows").len(), 0);
    let attention = env.api.call("attention.list", Value::Null).unwrap();
    let item = rows(&attention, "items")
        .iter()
        .find(|i| i["kind"] == "uncovered_sales")
        .unwrap_or_else(|| panic!("{attention:#}"));
    assert_eq!(item["count"], 2);
    assert_eq!(item["route"], "/carteira?aba=vendas");

    // 1) The purchase is recorded: the sale leaves the list by itself and gets its result.
    let vale = rows_[1]["asset_id"].as_str().unwrap().to_string();
    env.api
        .call(
            "portfolio.create_trade",
            json!({"asset_id": vale, "account_id": acc, "date": "2024-02-01", "side": "buy",
                   "quantity": "100", "price": "50", "cash_movement": false}),
        )
        .unwrap();
    let pending = env
        .api
        .call("portfolio.uncovered_sales", json!({}))
        .unwrap();
    assert_eq!(pending["open"], 1);
    let sales = env.api.call("portfolio.sales", json!({})).unwrap();
    assert_eq!(rows(&sales, "rows").len(), 1);
    assert_eq!(rows(&sales, "rows")[0]["result"], 100_000);

    // 2) The other one is accepted without a cost: it stops asking and stays out of the
    // result.
    let petr = rows(&pending, "rows")[0]["trade_id"].clone();
    let after = env
        .api
        .call(
            "portfolio.accept_uncovered_sale",
            json!({"trade_id": petr, "accepted": true}),
        )
        .unwrap();
    assert_eq!(after["open"], 0);
    assert_eq!(after["accepted"], 1);
    assert_eq!(rows(&after, "rows")[0]["accepted"], true);
    let attention = env.api.call("attention.list", Value::Null).unwrap();
    assert!(
        rows(&attention, "items")
            .iter()
            .all(|i| i["kind"] != "uncovered_sales")
    );
    assert_eq!(
        rows(&env.api.call("portfolio.sales", json!({})).unwrap(), "rows").len(),
        1
    );

    // It can be put back, and only a sale can be accepted.
    let back = env
        .api
        .call(
            "portfolio.accept_uncovered_sale",
            json!({"trade_id": petr, "accepted": false}),
        )
        .unwrap();
    assert_eq!(back["open"], 1);
    assert!(
        env.api
            .call(
                "portfolio.accept_uncovered_sale",
                json!({"trade_id": "nope", "accepted": true}),
            )
            .is_err()
    );

    // 3) Or the sale is deleted: nothing left to explain.
    env.api
        .call("portfolio.delete_trade", json!({"id": petr}))
        .unwrap();
    let pending = env
        .api
        .call("portfolio.uncovered_sales", json!({}))
        .unwrap();
    assert_eq!(pending["open"], 0);
    assert_eq!(rows(&pending, "rows").len(), 0);
}

/// The person still decides: a sale left out of the commit is not written.
#[test]
fn sale_without_history_can_be_left_out() {
    let env = Env::new();
    env.account("XP Investimentos", "brokerage");
    let m = mov(
        "Debito;10/06/2026;Transferência - Liquidação;VALE3 - VALE S.A.;XP INVESTIMENTOS CCTVM S/A;100;60,00;6.000,00\n",
    );
    let p = env.preview(json!({"files": [file("mov.csv", &m)]}));
    let index = rows(&p, "trades")[0]["index"].clone();
    let r = env
        .api
        .call("b3.commit", json!({"token": p["token"], "skip": [index]}))
        .unwrap();
    assert_eq!(r["trades"], 0);
    assert_eq!(r["pending_sales"], 0);
    assert_eq!(env.count("SELECT count(*) FROM trade"), 0);
}

// ---------------------------------------------------------------------------
// 3. Custodians → accounts

fn three_custodians() -> Vec<u8> {
    mov(
        "Credito;15/03/2026;Dividendo;ITSA4 - ITAUSA S.A.;NU INVESTIMENTOS S.A. - CTVM;100;0,10;10,00\n\
         Credito;15/04/2026;Dividendo;ITSA4 - ITAUSA S.A.;NU INVEST CORRETORA DE VALORES S.A.;100;0,10;10,00\n\
         Credito;15/04/2026;Dividendo;ITSA4 - ITAUSA S.A.;NU INVESTIMENTOS S.A. - CTVM;100;0,10;10,00\n\
         Credito;10/03/2026;APLICAÇÃO;RDB - RDB31497YWB;COOPERATIVA DE CREDITO VALE DO ITAJAI - VIACREDI;100000;0,01;1.000,00\n\
         Credito;12/03/2026;COMPRA / VENDA;CDB - CDBX1234567;ITAU UNIBANCO S.A.;1;1.000,00;1.000,00\n",
    )
}

#[test]
fn renamed_broker_is_one_custodian_and_each_gets_its_account() {
    let env = Env::new();
    let nu = env.account("NuInvest", "brokerage");
    env.account("Nubank", "checking");
    let via = env.account("Viacredi", "checking");
    let itau = env.account("Itaú", "checking");
    env.account("Vale Alimentação", "benefit");
    let p = env.preview(json!({"files": [file("mov.csv", &three_custodians())]}));
    let inst = rows(&p, "institutions");
    assert_eq!(inst.len(), 3, "{inst:#?}");
    let by = |id: &str| inst.iter().find(|i| i["account_id"] == id).unwrap();
    assert_eq!(by(&nu)["aliases"].as_array().unwrap().len(), 2);
    assert_eq!(by(&nu)["rows"], 3);
    assert_eq!(by(&nu)["match_reason"], "similar_name");
    assert_eq!(by(&via)["rows"], 1);
    assert_eq!(by(&itau)["account_name"], "Itaú");
    assert_eq!(p["counts"]["no_account"], 0);
    // The same dividend under the two names is one income.
    let incomes = rows(&p, "incomes");
    assert_eq!(
        incomes.iter().filter(|i| i["status"] == "new").count(),
        2,
        "{incomes:#?}"
    );
    assert_eq!(by(&nu)["summary"]["incomes"], 2);
    assert_eq!(by(&via)["summary"]["trades"], 1);
    assert_eq!(by(&via)["summary"]["assets"][0]["after"], "100000");
}

#[test]
fn accounts_created_by_the_import_and_the_saved_mapping() {
    let env = Env::new();
    let p = env.preview(json!({"files": [file("mov.csv", &three_custodians())]}));
    let inst = rows(&p, "institutions");
    assert_eq!(inst.len(), 3);
    let proposals: Vec<&Value> = inst.iter().map(|i| &i["proposed_new_account"]).collect();
    assert!(proposals.iter().all(|x| x.is_object()), "{proposals:#?}");
    let nu = &proposals[0];
    assert_eq!(nu["name"], "Nu Invest");
    assert_eq!(nu["kind"], "brokerage");
    assert_eq!(nu["institution_kind"], "broker");
    let via = proposals
        .iter()
        .find(|x| x["institution_name"] == "Viacredi")
        .unwrap();
    assert_eq!(via["name"], "Viacredi Investimentos");
    assert_eq!(via["kind"], "other");
    assert_eq!(via["opening_date"], "2026-03-09");

    let choices: Vec<Value> = inst
        .iter()
        .map(|i| json!({"institution": i["name"], "account_id": null, "new_account": i["proposed_new_account"]}))
        .collect();
    let p =
        env.preview(json!({"files": [file("mov.csv", &three_custodians())], "accounts": choices}));
    assert_eq!(p["counts"]["no_account"], 0);
    assert!(
        rows(&p, "institutions")
            .iter()
            .all(|i| i["match_reason"] == "created")
    );
    // The preview created nothing.
    assert_eq!(env.count("SELECT count(*) FROM account"), 0);
    let r = env
        .api
        .call("b3.commit", json!({"token": p["token"]}))
        .unwrap();
    assert_eq!(r["accounts_created"].as_array().unwrap().len(), 3);
    assert_eq!(r["errors"], json!([]));
    assert_eq!(env.count("SELECT count(*) FROM account"), 3);
    assert_eq!(
        env.count("SELECT count(*) FROM institution WHERE kind = 'broker'"),
        1
    );
    assert_eq!(env.count("SELECT count(*) FROM import_institution_map"), 3);

    // Next import: the saved mapping, and everything is already there.
    let p2 = env.preview(json!({"files": [file("mov.csv", &three_custodians())]}));
    assert!(
        rows(&p2, "institutions")
            .iter()
            .all(|i| i["mapped"] == true && i["match_reason"] == "saved")
    );
    assert!(
        rows(&p2, "incomes")
            .iter()
            .chain(rows(&p2, "trades"))
            .all(|x| x["status"] == "duplicate"),
        "{:#?} {:#?}",
        p2["incomes"],
        p2["trades"]
    );

    // Undoing the import removes the accounts and the mapping too.
    env.api
        .call("changes.undo", json!({"id": r["change_set_id"]}))
        .unwrap();
    assert_eq!(env.count("SELECT count(*) FROM account"), 0);
    assert_eq!(env.count("SELECT count(*) FROM import_institution_map"), 0);
}

#[test]
fn archived_account_invalidates_the_saved_mapping() {
    let env = Env::new();
    let nu = env.account("NuInvest", "brokerage");
    let m = mov(
        "Credito;15/03/2026;Dividendo;ITSA4 - ITAUSA S.A.;NU INVESTIMENTOS S.A. - CTVM;100;0,10;10,00\n",
    );
    let p = env.preview(json!({"files": [file("mov.csv", &m)]}));
    env.api
        .call("b3.commit", json!({"token": p["token"]}))
        .unwrap();
    env.api
        .call("accounts.update", json!({"id": nu, "archived": true}))
        .unwrap();
    let p = env.preview(json!({"files": [file("mov.csv", &m)]}));
    let i = &rows(&p, "institutions")[0];
    assert_eq!(i["mapped"], false);
    assert!(i["account_id"].is_null());
    assert!(i["proposed_new_account"].is_object());
}
