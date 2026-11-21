//! Tests for Food (Consumption) with made-up receipts (store, CNPJ, products, codes and amounts).

use serde_json::{Value, json};

use crate::api::Api;
use crate::paths::TreasuryPaths;

/// Made-up NFC-e DANFE: rice and coffee with GTIN, bananas on the scale, detergent (household).
pub(super) fn danfe(date: &str, coffee: &str, paid: &str) -> String {
    format!(
        "MERCADO FICTICIO DO TESTE LTDA
CNPJ: 11.222.333/0001-81 Rua Inventada, 10 - Cidade Exemplo - PR
DANFE NFC-e - Documento Auxiliar da Nota Fiscal de Consumidor Eletrônica
Código Descrição Qtde Un Vl Unit Vl Total
1 7890000000017 ARROZ EXEMPLO TIPO1 5KG 1 UN 25,00 25,00
2 7890000000024 CAFE EXEMPLO TORRADO 500G 1 UN {coffee} {coffee}
3 2000123 BANANA PRATA 1 KG 6,00 6,00
4 7890000000031 DETERGENTE EXEMPLO 500ML 2 UN 2,50 5,00
5 7890000000048 REFRIGERANTE EXEMPLO 2L 1 UN 9,00 9,00
Qtde. total de itens 5
Valor total R$ {paid}
Valor a Pagar R$ {paid}
Emissão: {date} 18:02:11
"
    )
}

pub(super) fn api() -> (tempfile::TempDir, Api, String, String) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana", "Bia"]}),
    )
    .unwrap();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Conta", "kind": "checking", "opening_date": "2026-01-01"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let cats = api.call("categories.list", json!({})).unwrap();
    let find = |name: &str| -> String {
        cats.as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .unwrap_or_else(|| panic!("{name}"))["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    (dir, api, acc, find("Mercado") + "|" + &find("Restaurantes"))
}

fn txn(api: &Api, acc: &str, date: &str, amount: i64, cat: &str, desc: &str) -> Value {
    api.call(
        "transactions.create",
        json!({"account_id": acc, "date": date, "amount": amount, "description": desc,
               "kind": "expense", "category_id": cat}),
    )
    .unwrap()
}

#[test]
fn products_prices_classes_and_the_food_summary() {
    let (_d, api, acc, cats) = api();
    let (market, restaurant) = cats.split_once('|').unwrap();
    // Two purchases at the same market: coffee goes up from 20.00 to 24.00.
    let t1 = txn(&api, &acc, "2026-09-05", -6_500, market, "Mercado fictício");
    let t2 = txn(&api, &acc, "2026-10-05", -6_900, market, "Mercado fictício");
    let r1 = api
        .call(
            "receipts.import",
            json!({"text": danfe("05/09/2026", "20,00", "65,00"), "txn_id": t1["id"]}),
        )
        .unwrap();
    api.call(
        "receipts.import",
        json!({"text": danfe("05/10/2026", "24,00", "69,00"), "txn_id": t2["id"]}),
    )
    .unwrap();
    // The items become products (the same coffee in both purchases, by GTIN).
    let items = &r1["items"];
    assert!(items[0]["product_id"].is_string(), "{r1}");
    assert_eq!(items[0]["gtin"], "7890000000017");
    // Bananas on the scale: code 2… is not a GTIN; price per kg.
    assert_eq!(items[2]["gtin"], Value::Null);
    assert_eq!(items[2]["base_unit"], "kg");
    assert_eq!(items[2]["base_price"], 600);
    // Rice 5 kg at 25.00: 5.00 per kg.
    assert_eq!(items[0]["base_price"], 500);
    let products = api.call("food.products", json!({"food": true})).unwrap();
    let coffee = products
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"].as_str().unwrap().starts_with("Cafe"))
        .unwrap()
        .clone();
    assert_eq!(coffee["purchases"], 2);
    // 500 g: 40.00 per kg and then 48.00 per kg (+20%).
    assert_eq!(coffee["first_price"], 4_000);
    assert_eq!(coffee["last_price"], 4_800);
    assert!((coffee["change"].as_f64().unwrap() - 0.2).abs() < 1e-9);
    assert_eq!(
        coffee["category_path"],
        "Alimentação › Mercearia › Açúcar e café"
    );
    assert_eq!(coffee["effective_class"], "essential");
    // The detergent is "house and hygiene".
    let home = api.call("food.products", json!({"food": false})).unwrap();
    assert!(
        home.as_array()
            .unwrap()
            .iter()
            .any(|p| p["name"].as_str().unwrap().starts_with("Detergente"))
    );
    let h = api
        .call("food.product_history", json!({"id": coffee["id"]}))
        .unwrap();
    assert_eq!(h["points"].as_array().unwrap().len(), 2);

    // Eating out: restaurant; a market purchase without a receipt also counts as home.
    txn(
        &api,
        &acc,
        "2026-10-12",
        -8_000,
        restaurant,
        "Restaurante inventado",
    );
    txn(&api, &acc, "2026-10-20", -3_000, market, "Feira sem cupom");
    let s = api
        .call("food.summary", json!({"month": "2026-10"}))
        .unwrap();
    // Receipt foods: 69.00 − 5.00 detergent = 64.00; plus 30.00 without a receipt.
    assert_eq!(s["receipt_items"], 6_400, "{s}");
    assert_eq!(s["market_without_receipt"], 3_000);
    assert_eq!(s["home"], 9_400);
    assert_eq!(s["out"], 8_000);
    assert_eq!(s["people"], 2);
    assert_eq!(s["per_person"], (9_400 + 8_000) / 2);
    let classes = s["classes"].as_array().unwrap();
    let sup = classes
        .iter()
        .find(|c| c["class"] == "superfluous")
        .unwrap();
    assert_eq!(sup["spent"], 900);
    assert_eq!(s["months"].as_array().unwrap().len(), 6);
    let sep = s["months"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["month"] == "2026-09")
        .unwrap();
    assert_eq!(sep["home"], 6_000);
    assert_eq!(sep["non_food"], 500);
    // The detergent is in the Home and cleaning segment; the rest, in Food.
    let seg = |key: &str| {
        s["segments"]
            .as_array()
            .unwrap()
            .iter()
            .find(|g| g["seed_key"] == key)
            .unwrap_or_else(|| panic!("{key}: {s}"))
            .clone()
    };
    assert_eq!(seg("seg.casa_limpeza")["spent"], 500);
    assert_eq!(seg("seg.casa_limpeza")["previous"], 500);
    assert_eq!(seg("seg.alimentacao")["spent"], 6_400);
    assert_eq!(s["total"], 6_900);
    assert_eq!(s["previous_total"], 6_500);
    // The coffee went up in price and was bought in the month.
    assert!(
        s["risers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["name"].as_str().unwrap().starts_with("Cafe"))
    );
    let home = api
        .call(
            "food.summary",
            json!({"month": "2026-10", "segment_id": seg("seg.casa_limpeza")["category_id"]}),
        )
        .unwrap();
    assert_eq!(home["categories"][0]["name"], "Limpeza", "{home}");
    assert_eq!(home["categories"][0]["spent"], 500);
    assert_eq!(home["top_products"].as_array().unwrap().len(), 1);

    // The person changes the soda's class and merges products.
    let soda = products
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"].as_str().unwrap().starts_with("Refrigerante"))
        .unwrap()
        .clone();
    assert_eq!(soda["effective_class"], "superfluous");
    api.call(
        "food.product_update",
        json!({"id": soda["id"], "class": "complementary"}),
    )
    .unwrap();
    let s = api
        .call("food.summary", json!({"month": "2026-10"}))
        .unwrap();
    assert!(
        !s["classes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["class"] == "superfluous")
    );
    let rice = products
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"].as_str().unwrap().starts_with("Arroz"))
        .unwrap()
        .clone();
    api.call(
        "food.product_merge",
        json!({"from_id": soda["id"], "into_id": rice["id"]}),
    )
    .unwrap();
    let after = api.call("food.products", json!({})).unwrap();
    assert!(
        !after
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["id"] == soda["id"])
    );
}

#[test]
fn categories_tree_and_payments() {
    let (_d, api, acc, cats) = api();
    let (market, _) = cats.split_once('|').unwrap();
    let t1 = txn(&api, &acc, "2026-10-05", -4_000, market, "Cartão");
    let t2 = txn(&api, &acc, "2026-10-05", -2_900, market, "Vale-alimentação");
    let r = api
        .call(
            "receipts.import",
            json!({"text": danfe("05/10/2026", "24,00", "69,00")}),
        )
        .unwrap();
    // Paid in two ways: the payments must add up to the receipt.
    assert!(
        api.call(
            "food.set_payments",
            json!({"receipt_id": r["id"], "payments": [{"txn_id": t1["id"], "amount": 4_000}]}),
        )
        .is_err()
    );
    api.call(
        "food.set_payments",
        json!({"receipt_id": r["id"], "payments": [
            {"txn_id": t1["id"], "amount": 4_000}, {"txn_id": t2["id"], "amount": 2_900}]}),
    )
    .unwrap();
    let s = api
        .call("food.summary", json!({"month": "2026-10"}))
        .unwrap();
    assert_eq!(s["receipts_without_txn"], 0);
    // Transactions with a receipt do not count again as a market without a receipt.
    assert_eq!(s["market_without_receipt"], 0);
    assert_eq!(s["home"], 6_400);

    // Tree: roots from the seed with subcategories and inherited class.
    let tree = api.call("food.categories", json!({})).unwrap();
    let list = tree.as_array().unwrap();
    let fruit = list
        .iter()
        .find(|c| c["seed_key"] == "hortifruti.frutas")
        .unwrap();
    assert_eq!(fruit["effective_class"], "essential");
    assert_eq!(fruit["class"], Value::Null);
    assert_eq!(fruit["depth"], 2);
    let c = api
        .call(
            "food.category_create",
            json!({"parent_id": fruit["id"], "name": "Frutas importadas", "class": "superfluous"}),
        )
        .unwrap();
    assert_eq!(c["effective_class"], "superfluous");
    assert_eq!(
        c["path"],
        "Alimentação › Hortifrúti › Frutas › Frutas importadas"
    );
    // It cannot sit inside itself.
    assert!(
        api.call(
            "food.category_update",
            json!({"id": fruit["id"], "parent_id": c["id"]}),
        )
        .is_err()
    );
    api.call("food.category_delete", json!({"id": c["id"]}))
        .unwrap();
    // Segments are the roots; Pet, Cigarettes and Others remain roots.
    let tree = api.call("food.categories", json!({})).unwrap();
    let by_key = |k: &str| {
        tree.as_array()
            .unwrap()
            .iter()
            .find(|c| c["seed_key"] == k)
            .unwrap_or_else(|| panic!("{k}"))
            .clone()
    };
    assert_eq!(by_key("seg.vestuario")["depth"], 0);
    assert_eq!(by_key("pet")["depth"], 0);
    assert_eq!(by_key("limpeza")["path"], "Casa e limpeza › Limpeza");
    assert_eq!(by_key("farmacia")["path"], "Saúde › Farmácia");
    assert_eq!(by_key("pet.racao")["path"], "Pet › Ração e petiscos");
    assert_eq!(
        by_key("vestuario.roupas")["effective_class"],
        "complementary"
    );
}

#[test]
fn segments_reach_old_vaults_without_moving_what_the_person_moved() {
    let (_d, api, _, _) = api();
    api.with_session(|s, _| {
        let c = s.db.conn();
        // V135 vault: only the receipt groups, and the person put Hygiene inside Pharmacy.
        c.execute(
            "UPDATE product_category SET parent_id = NULL WHERE seed_key NOT LIKE '%.%'",
            [],
        )?;
        c.execute("DELETE FROM product_category WHERE seed_key LIKE 'seg.%'", [])?;
        c.execute(
            "DELETE FROM product_category WHERE seed_key IN ('pet.racao', 'pet.cuidados')",
            [],
        )?;
        c.execute(
            "UPDATE product_category SET parent_id = (SELECT id FROM product_category WHERE seed_key = 'farmacia')
              WHERE seed_key = 'higiene'",
            [],
        )?;
        crate::settings::set(c, "food.seeded_segments", &false)?;
        super::on_open(c)?;
        let parent = |k: &str| -> rusqlite::Result<Option<String>> {
            c.query_row(
                "SELECT p.seed_key FROM product_category c LEFT JOIN product_category p ON p.id = c.parent_id
                  WHERE c.seed_key = ?1",
                [k],
                |r| r.get(0),
            )
        };
        assert_eq!(parent("mercearia")?.as_deref(), Some("seg.alimentacao"));
        assert_eq!(parent("bebe")?.as_deref(), Some("seg.saude"));
        assert_eq!(parent("higiene")?.as_deref(), Some("farmacia"));
        assert_eq!(parent("pet.racao")?.as_deref(), Some("pet"));
        // Only once: the segment deleted by the person does not come back.
        c.execute("DELETE FROM product_category WHERE seed_key = 'seg.lazer'", [])?;
        super::on_open(c)?;
        assert!(parent("lazer.brinquedos").is_err());
        Ok(())
    })
    .unwrap();
}

#[test]
fn manual_purchase_in_any_country() {
    let (_d, api, acc, _) = api();
    api.call("country.set", json!({"country": "PT"})).unwrap();
    let tree = api.call("food.categories", json!({})).unwrap();
    let cat = |k: &str| {
        tree.as_array()
            .unwrap()
            .iter()
            .find(|c| c["seed_key"] == k)
            .unwrap()["id"]
            .clone()
    };
    // Street market and clothes store without a receipt: the transaction is created along with it.
    let r = api
        .call(
            "food.purchase_save",
            json!({"date": "2026-10-10", "store_name": "Feira do Bairro Inventada",
                   "items": [
                       {"description": "Tomate", "quantity": "1,5", "unit": "kg", "unit_price": 600, "total": 900},
                       {"description": "Camiseta básica", "quantity": "2", "unit": "un", "total": 6_000},
                       {"description": "Vaso de barro", "total": 1_500, "category_id": cat("decoracao.decoracao")}
                   ],
                   "discount": 400,
                   "payment": {"mode": "create", "account_id": acc}}),
        )
        .unwrap();
    assert_eq!(r["origin"], "manual");
    assert_eq!(r["paid"], 8_000);
    // Outside Brazil there is no estimated tax.
    assert_eq!(r["tax"], 0);
    let items = r["items"].as_array().unwrap();
    assert_eq!(items[0]["base_unit"], "kg");
    assert_eq!(items[0]["base_price"], 600);
    let txn = api
        .call("transactions.get", json!({"id": r["txn_id"]}))
        .unwrap();
    assert_eq!(txn["amount"], -8_000, "{txn}");
    // The T-shirt falls on its own into Clothes; the vase, into the chosen category.
    let products = api.call("food.products", json!({})).unwrap();
    let path = |name: &str| {
        products
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["name"].as_str().unwrap().starts_with(name))
            .unwrap_or_else(|| panic!("{name}"))["category_path"]
            .clone()
    };
    assert_eq!(path("Camiseta"), "Vestuário e calçados › Roupas");
    assert_eq!(path("Vaso"), "Casa e decoração › Decoração");
    assert_eq!(
        path("Tomate"),
        "Alimentação › Hortifrúti › Legumes e verduras"
    );

    let s = api
        .call("food.summary", json!({"month": "2026-10"}))
        .unwrap();
    assert_eq!(s["total"], 8_000, "{s}");
    let clothes = s["segments"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["seed_key"] == "seg.vestuario")
        .unwrap()
        .clone();
    // 60.00 minus the discount share (400 × 6000/8400).
    assert_eq!(clothes["spent"], 6_000 - 286);
    let list = api
        .call(
            "food.purchases",
            json!({"from": "2026-10-01", "to": "2026-10-31", "segment_id": clothes["category_id"]}),
        )
        .unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);
    let none = api
        .call(
            "food.purchases",
            json!({"from": "2026-10-01", "to": "2026-10-31", "query": "padaria"}),
        )
        .unwrap();
    assert!(none.as_array().unwrap().is_empty());
    assert_eq!(
        api.call("food.stores", json!({})).unwrap()[0],
        "Feira do Bairro Inventada"
    );

    // Edit: the transaction created along with it follows the amount.
    let r2 = api
        .call(
            "food.purchase_save",
            json!({"id": r["id"], "date": "2026-10-11", "store_name": "Feira do Bairro Inventada",
                   "items": [{"description": "Tomate", "quantity": "2", "unit": "kg", "total": 1_200}],
                   "payment": {"mode": "link", "txn_id": r["txn_id"]}}),
        )
        .unwrap();
    assert_eq!(r2["id"], r["id"]);
    assert_eq!(r2["item_count"], 1);
    let txn = api
        .call("transactions.get", json!({"id": r["txn_id"]}))
        .unwrap();
    assert_eq!(txn["amount"], -1_200);
    assert_eq!(txn["date"], "2026-10-11");
    // The same tomato as before (by name), now at 6.00 per kg again.
    let tomato = api
        .call("food.products", json!({"query": "tomate"}))
        .unwrap();
    assert_eq!(tomato.as_array().unwrap().len(), 1);
    assert_eq!(tomato[0]["last_unit"], "kg");

    // Link to an existing transaction: the purchase can itemize only part of it, and more than one
    // purchase can use the same transaction; the sum never exceeds the amount.
    let other = api
        .call(
            "transactions.create",
            json!({"account_id": acc, "date": "2026-10-12", "amount": -2_500,
                   "description": "Loja de brinquedos inventada", "kind": "expense"}),
        )
        .unwrap();
    let toy = json!({"date": "2026-10-12", "store_name": "Loja de brinquedos inventada",
                     "items": [{"description": "Boneca de pano", "total": 2_600}],
                     "payment": {"mode": "link", "txn_id": other["id"]}});
    let e = api.call("food.purchase_save", toy.clone()).unwrap_err();
    assert!(e.message().contains("passa do que ainda cabe"), "{e}");
    // Candidates: the transaction with an equal amount, and also the largest one the purchase fits.
    let c = api
        .call(
            "food.purchase_candidates",
            json!({"date": "2026-10-11", "amount": 1_000}),
        )
        .unwrap();
    assert_eq!(c[0]["txn_id"], other["id"]);
    let mut part = toy.clone();
    part["items"][0]["total"] = json!(1_500);
    let t = api.call("food.purchase_save", part.clone()).unwrap();
    assert_eq!(t["paid"], 1_500);
    // The transaction does not change amount: the remaining R$ 10.00 stay without detail.
    let kept = api
        .call("transactions.get", json!({"id": other["id"]}))
        .unwrap();
    assert_eq!(kept["amount"], -2_500);
    assert_eq!(kept["receipt_id"], t["id"]);
    // Another purchase on the same transaction fits up to what is left (R$ 10.00), no more.
    let mut second = toy.clone();
    second["items"] = json!([{"description": "Carrinho de madeira", "total": 1_100}]);
    assert!(api.call("food.purchase_save", second.clone()).is_err());
    second["items"][0]["total"] = json!(1_000);
    let t2 = api.call("food.purchase_save", second).unwrap();
    assert!(
        api.call(
            "food.purchase_candidates",
            json!({"date": "2026-10-11", "amount": 100}),
        )
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .all(|c| c["txn_id"] != other["id"]),
        "fully itemized transaction is no longer a candidate"
    );
    api.call("food.purchase_delete", json!({"id": t2["id"]}))
        .unwrap();
    // Deleting the linked purchase keeps the transaction; the one that created it removes it too.
    api.call("food.purchase_delete", json!({"id": t["id"]}))
        .unwrap();
    assert!(
        api.call("transactions.get", json!({"id": other["id"]}))
            .is_ok()
    );
    api.call("food.purchase_delete", json!({"id": r["id"]}))
        .unwrap();
    assert!(
        api.call("transactions.get", json!({"id": r["txn_id"]}))
            .is_err()
    );
    // A receipt read from the QR is not edited here.
    assert!(
        api.call(
            "receipts.import",
            json!({"text": danfe("05/10/2026", "24,00", "69,00")})
        )
        .is_err(),
        "the NFC-e is Brazilian"
    );
}

#[test]
fn manual_purchase_without_txn_stays_apart_and_brazil_estimates_the_tax() {
    let (_d, api, _, _) = api();
    let r = api
        .call(
            "food.purchase_save",
            json!({"date": "2026-10-10", "items": [{"description": "Arroz exemplo 5kg", "total": 2_500}],
                   "payment": {"mode": "none"}}),
        )
        .unwrap();
    assert_eq!(r["txn_id"], Value::Null);
    // In Brazil, the tax estimated by the consumption group's percentage.
    assert!(r["tax"].as_i64().unwrap() > 0, "{r}");
    assert_eq!(r["store_name"], Value::Null);
    let s = api
        .call("food.summary", json!({"month": "2026-10"}))
        .unwrap();
    assert_eq!(s["total"], 0);
    assert_eq!(s["receipts_without_txn"], 1);
    assert_eq!(s["receipts_without_txn_amount"], 2_500);
    // Invalid unit, quantity and amounts.
    for bad in [
        json!([{"description": "X", "total": 0}]),
        json!([{"description": "X", "total": 100, "unit": "caixa"}]),
        json!([{"description": "X", "total": 100, "quantity": "-1"}]),
        json!([{"description": " ", "total": 100}]),
    ] {
        assert!(
            api.call(
                "food.purchase_save",
                json!({"date": "2026-10-10", "items": bad, "payment": {"mode": "none"}}),
            )
            .is_err()
        );
    }
}

#[test]
fn gtin_check_digit_and_old_receipts_are_linked_on_open() {
    use crate::receipts::nfce::valid_gtin;
    assert_eq!(
        valid_gtin("7890000000017").as_deref(),
        Some("7890000000017")
    );
    assert_eq!(valid_gtin("7890000000018"), None);
    // Codes starting with 2: scale and internal store use.
    assert_eq!(valid_gtin("2000000000015"), None);
    assert_eq!(valid_gtin("12345"), None);
    let xml = "<infNFe><det><prod><cProd>123</cProd><cEAN>7890000000024</cEAN>\
               <xProd>CAFE EXEMPLO 500G</xProd><qCom>1.0000</qCom><uCom>UN</uCom>\
               <vUnCom>20.00</vUnCom><vProd>20.00</vProd></prod></det></infNFe>";
    let p = crate::receipts::nfce::parse_xml(xml);
    assert_eq!(p.items[0].gtin.as_deref(), Some("7890000000024"));

    // Receipt from before V135 (items without a product): gets the products when the vault opens.
    let (_d, api, _, _) = api();
    api.call(
        "receipts.import",
        json!({"text": danfe("05/10/2026", "24,00", "69,00")}),
    )
    .unwrap();
    api.with_session(|s, _| {
        s.db.conn()
            .execute("UPDATE receipt_item SET product_id = NULL", [])?;
        s.db.conn().execute("DELETE FROM product", [])?;
        assert_eq!(super::on_open(s.db.conn())?, 5);
        assert_eq!(super::on_open(s.db.conn())?, 0);
        Ok(())
    })
    .unwrap();
    let s = api
        .call("food.summary", json!({"month": "2026-10"}))
        .unwrap();
    assert_eq!(s["unlinked_items"], 0);
}

#[test]
fn receipt_noise_leaves_the_product_name() {
    use super::clean_description as c;
    assert_eq!(c("51354 Camiseta Gg 1,00 Un +-53,48"), "Camiseta Gg");
    assert_eq!(c("001 7891234567895 ARROZ TIPO 1 5KG"), "ARROZ TIPO 1 5KG");
    assert_eq!(
        c("002 BANANA PRATA KG 0,734 KG X 6,99 5,13"),
        "BANANA PRATA KG"
    );
    assert_eq!(
        c("LEITE INTEGRAL 1L 2 UN X 5,49 10,98"),
        "LEITE INTEGRAL 1L"
    );
    assert_eq!(c("DETERGENTE 500ML  2,49G"), "DETERGENTE 500ML");
    assert_eq!(c("Café torrado 500g"), "Café torrado 500g");
    assert_eq!(c("12345"), "12345");
    assert_eq!(
        super::product_name("51354 CAMISETA GG 1,00 UN +-53,48"),
        "Camiseta Gg"
    );
}

/// Made-up text receipt with the given lines ("code description qty unit price total").
fn receipt_text(date: &str, lines: &[&str], paid: &str) -> String {
    format!(
        "MERCADO FICTICIO DO TESTE LTDA
CNPJ: 11.222.333/0001-81 Rua Inventada, 10 - Cidade Exemplo - PR
DANFE NFC-e - Documento Auxiliar da Nota Fiscal de Consumidor Eletrônica
Código Descrição Qtde Un Vl Unit Vl Total
{}
Valor total R$ {paid}
Valor a Pagar R$ {paid}
Emissão: {date} 18:02:11
",
        lines.join("\n")
    )
}

#[test]
fn shrinkflation_health_and_diet() {
    let (_d, api, _acc, _cats) = api();
    // Made-up GTINs with the correct check digit.
    api.call(
        "receipts.import",
        json!({"text": receipt_text("10/01/2026", &[
            "1 7890000000055 CAFE EXEMPLO TORRADO 500G 1 UN 20,00 20,00",
            "2 7890000000062 BISCOITO EXEMPLO RECHEADO 130G 2 UN 4,00 8,00",
            "3 7890000000079 OVOS EXEMPLO BRANCOS 12UN 1 UN 12,00 12,00",
        ], "40,00")}),
    )
    .unwrap();
    // The same coffee line at 450 g, more expensive per kg: shrinkflation.
    api.call(
        "receipts.import",
        json!({"text": receipt_text("10/02/2026", &[
            "1 7890000000086 CAFE EXEMPLO TORRADO 450G 1 UN 20,00 20,00",
        ], "20,00")}),
    )
    .unwrap();
    let alerts = api.call("ai.alerts.list", json!({})).unwrap();
    assert!(
        alerts.to_string().contains("shrinkflation"),
        "shrinkflation alert: {alerts}"
    );
    let products = api.call("food.products", json!({})).unwrap();
    let id_of = |part: &str| -> String {
        products
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["name"].as_str().unwrap().contains(part))
            .unwrap_or_else(|| panic!("{part}"))["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let cafe450 = id_of("450");
    let h = api
        .call("food.product_history", json!({"id": cafe450}))
        .unwrap();
    assert_eq!(h["same_line"].as_array().unwrap().len(), 1, "{h}");
    assert!(h["health"].is_null());

    // Open Food Facts responses recorded as if the source had looked them up (made up).
    api.with_session(|s, _| {
        let c = s.db.conn();
        for (g, body) in [
            (
                "07890000000062",
                r#"{"status":1,"product":{"product_name":"Biscoito Exemplo","quantity":"130 g","nova_group":4,
                   "allergens_tags":["en:gluten","en:milk"],"nutriments":{"added-sugars_100g":30,
                   "saturated-fat_100g":9,"sodium_100g":0.3,"proteins_100g":6}}}"#,
            ),
            (
                "07890000000079",
                r#"{"status":1,"product":{"product_name":"Ovos Exemplo","quantity":"12","nova_group":1,
                   "allergens_tags":["en:eggs"],"nutriments":{"proteins_100g":13}}}"#,
            ),
        ] {
            super::open_data::store(c, &super::open_data::parse(g, body, "2026-02-11T10:00:00.000Z"))?;
        }
        Ok(())
    })
    .unwrap();
    let bia = api.call("household.list_members", json!({})).unwrap()[1]["id"]
        .as_str()
        .unwrap()
        .to_string();
    api.call(
        "food.diet_set",
        json!({"member_id": bia, "restrictions": ["gluten"]}),
    )
    .unwrap();
    assert!(
        api.call(
            "food.diet_set",
            json!({"member_id": bia, "restrictions": ["sorvete"]})
        )
        .is_err()
    );
    let today = chrono::NaiveDate::from_ymd_opt(2026, 2, 20).unwrap();
    let s = api
        .with_session(|s, _| super::health::summary(s.db.conn(), today))
        .unwrap();
    assert!(!s.source_enabled, "Open Food Facts starts off");
    assert!(s.allergens.iter().any(|a| a.member_id == bia
        && a.restriction == "gluten"
        && a.product_name.to_lowercase().contains("biscoito")));
    let h = api
        .call("food.product_history", json!({"id": id_of("Biscoito")}))
        .unwrap();
    let high = h["health"]["high_in"].to_string();
    assert!(
        high.contains("added_sugar") && high.contains("saturated_fat"),
        "{h}"
    );
    assert_eq!(h["health"]["data"]["nova"], 4);
}

#[test]
fn barcode_typed_by_hand_on_the_product_and_on_the_purchase() {
    let (_d, api, _acc, _cats) = api();
    let first = api
        .call(
            "food.purchase_save",
            json!({"date": "2026-10-10", "store_name": "Mercadinho Inventado",
                   "items": [{"description": "Café Exemplo 500g", "total": 2_000},
                             {"description": "Leite Exemplo 1L", "total": 600}],
                   "payment": {"mode": "none"}}),
        )
        .unwrap();
    let id_of = |part: &str| -> String {
        api.call("food.products", json!({}))
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["name"].as_str().unwrap().contains(part))
            .unwrap_or_else(|| panic!("{part}"))["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let cafe = id_of("Café");
    // Made-up GTIN with the correct check digit; spaces and dash do not get in the way.
    let p = api
        .call(
            "food.product_gtin_add",
            json!({"product_id": cafe, "code": "789 0000 00001-7"}),
        )
        .unwrap();
    assert_eq!(p["gtins"], json!(["7890000000017"]));
    assert_eq!(p["gtin"], "7890000000017");
    // The errors say what is wrong.
    let err = |code: &str| {
        api.call(
            "food.product_gtin_add",
            json!({"product_id": cafe, "code": code}),
        )
        .unwrap_err()
        .message()
    };
    let (size, digit, internal) = (err("78900"), err("7890000000018"), err("2000000000015"));
    assert!(size != digit && digit != internal && size != internal);
    let leite = id_of("Leite");
    let other = api
        .call(
            "food.product_gtin_add",
            json!({"product_id": leite, "code": "7890000000017"}),
        )
        .unwrap_err()
        .message();
    assert!(other.contains("Café"), "{other}");
    // On the purchase, the code links the item to the product even with another name.
    let r = api
        .call(
            "food.purchase_save",
            json!({"date": "2026-10-20", "store_name": "Mercadinho Inventado",
                   "items": [{"description": "CAFE EXEMPLO TRAD", "total": 2_100,
                              "gtin": "7890000000017"}],
                   "payment": {"mode": "none"}}),
        )
        .unwrap();
    let got = api.call("receipts.get", json!({"id": r["id"]})).unwrap();
    assert_eq!(got["items"][0]["product_id"], cafe.as_str());
    assert_eq!(got["items"][0]["gtin"], "7890000000017");
    // Invalid code on the purchase: nothing is written.
    assert!(
        api.call(
            "food.purchase_save",
            json!({"date": "2026-10-21", "items": [{"description": "X", "total": 100,
                   "gtin": "7890000000018"}], "payment": {"mode": "none"}}),
        )
        .is_err()
    );
    api.call(
        "food.product_gtin_remove",
        json!({"product_id": cafe, "code": "7890000000017"}),
    )
    .unwrap();
    let h = api
        .call("food.product_history", json!({"id": cafe}))
        .unwrap();
    assert_eq!(h["product"]["gtins"], json!([]));
    let _ = first;
}

/// A market transaction itemized only in part: the items count as food at home and the
/// rest keeps counting as market without a receipt (the sum is the transaction's amount).
#[test]
fn partly_detailed_market_txn_keeps_the_rest_as_market_without_receipt() {
    let (_d, api, acc, cats) = api();
    let mercado = cats.split('|').next().unwrap().to_string();
    let today = crate::time::fmt_date(crate::time::today_local());
    let t = txn(&api, &acc, &today, -30_000, &mercado, "Mercado Inventado");
    api.call(
        "food.purchase_save",
        json!({"date": today, "store_name": "Mercado Inventado",
               "items": [{"description": "Patinho moído", "quantity": "1", "unit": "kg", "total": 4_000},
                         {"description": "Café torrado 500g", "total": 2_000}],
               "payment": {"mode": "link", "txn_id": t["id"]}}),
    )
    .unwrap();
    let s = api
        .call("food.summary", json!({"month": &today[..7]}))
        .unwrap();
    assert_eq!(s["market_without_receipt"], 24_000);
}

/// Items pasted as text: the same line reader as the photo, without OCR.
#[test]
fn pasted_text_becomes_purchase_lines() {
    let (_d, api, _, _) = api();
    let r = api
        .call(
            "receipts.read_text",
            json!({"text": "Arroz tipo 1 5kg 24,90\nFeijao carioca 1kg 8,49\n\nCafe torrado 500g 18,00\nTOTAL 51,39"}),
        )
        .unwrap();
    assert_eq!(r["engine"], "text");
    let items = r["receipt"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 3, "{r}");
    assert_eq!(items[0]["total"], 2_490);
    assert_eq!(r["reading"]["total"], 5_139);
    assert_ne!(r["tier"], "unbalanced");
    assert!(
        api.call("receipts.read_text", json!({"text": "  "}))
            .is_err()
    );
}

/// US receipt (made-up text): tax is added at the end, the letters mark the taxable items,
/// weight comes in pounds and descriptions in abbreviated English.
const US_RECEIPT: &str = "FRESH MARKET EXEMPLO #123
123 MAIN ST SPRINGFIELD
11/14/2026 05:42 PM
GV WHL MLK GAL          3.48 F
ORG BNNS
2.31 lb @ 0.59/lb       1.36 F
GRND BF 80/20 2LB       9.98 F
PAPER TOWELS 6 ROLLS   12.00 T
DISH SOAP 24 FL OZ      3.00 T
MFR COUPON              1.00-
SUBTOTAL               28.82
TAX 1 7.250%            1.09
TOTAL                  29.91
VISA TEND              29.91
ITEMS SOLD 5";

#[test]
fn us_receipt_adds_the_sales_tax_at_the_end() {
    let (_d, api, acc, _) = api();
    api.call("country.set", json!({"country": "US"})).unwrap();
    let profile = api.call("food.profile", json!({})).unwrap();
    assert_eq!(profile["tax"], "added");
    assert_eq!(profile["units"], "us");

    // Reading: items = subtotal, subtotal + tax = total; both sums add up.
    let r = api
        .call("receipts.read_text", json!({"text": US_RECEIPT}))
        .unwrap();
    assert_eq!(r["tier"], "checked", "{r}");
    assert_eq!(r["reading"]["tax"], 109);
    assert_eq!(r["reading"]["subtotal"], 2_882);
    assert_eq!(r["reading"]["discount"], 100);
    assert_eq!(r["reading"]["total"], 2_991);
    let items = r["reading"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 5, "{r}");
    assert_eq!(items[0]["taxable"], false);
    assert_eq!(items[3]["taxable"], true);
    // Variable weight in pounds, on the line below the item.
    assert_eq!(items[1]["quantity"], "2.31");
    assert_eq!(items[1]["total"], 136);
    assert_eq!(r["receipt"]["tax_total"], 109);
    assert_eq!(r["receipt"]["date"], "2026-11-14");

    // Category predicted from the English words.
    let cats = api.call("food.categories", json!({})).unwrap();
    let key_of = |id: &Value| {
        cats.as_array()
            .unwrap()
            .iter()
            .find(|c| &c["id"] == id)
            .unwrap()["seed_key"]
            .clone()
    };
    let guess = |d: &str| api.call("food.guess", json!({"description": d})).unwrap();
    assert_eq!(
        key_of(&guess("GV WHL MLK GAL")["category_id"]),
        "laticinios"
    );
    assert_eq!(key_of(&guess("ORG BNNS")["category_id"]), "hortifruti");
    assert_eq!(guess("GV WHL MLK GAL")["reason"], "words");

    // Saving: the amount paid is items − discount + tax, and tax goes only to the taxable ones.
    let t = txn(&api, &acc, "2026-11-14", -2_991, "", "FRESH MARKET EXEMPLO");
    let p = api
        .call(
            "food.purchase_save",
            json!({"date": "2026-11-14", "store_name": "Fresh Market Exemplo",
                   "items": [
                       {"description": "GV WHL MLK GAL", "total": 348, "taxable": false},
                       {"description": "ORG BNNS", "quantity": "2.31", "unit": "lb", "total": 136, "taxable": false},
                       {"description": "GRND BF 80/20 2LB", "total": 998, "taxable": false},
                       {"description": "PAPER TOWELS 6 ROLLS", "total": 1_200, "taxable": true},
                       {"description": "DISH SOAP 24 FL OZ", "total": 300, "taxable": true}],
                   "discount": 100, "tax": 109,
                   "payment": {"mode": "link", "txn_id": t["id"]}}),
        )
        .unwrap();
    assert_eq!(p["paid"], 2_991);
    assert_eq!(p["tax_added"], 109);
    assert_eq!(p["tax"], 109);
    assert_eq!(p["tax_printed"], true);
    let full = api.call("receipts.get", json!({"id": p["id"]})).unwrap();
    let it = full["items"].as_array().unwrap();
    assert_eq!(it[0]["tax"], 0);
    assert_eq!(
        it[3]["tax"].as_i64().unwrap() + it[4]["tax"].as_i64().unwrap(),
        109
    );
    assert_eq!(it[3]["tax_origin"], "nfce", "printed on the receipt");
    // Unit price stays in kg and L, without tax: bananas at 0.59/lb = 1.30/kg.
    assert_eq!(it[1]["base_unit"], "kg");
    assert_eq!(it[1]["base_price"], 130);
    // Milk: 3.48 per gallon = 0.92 per liter.
    assert_eq!(it[0]["base_unit"], "l");
    assert_eq!(it[0]["base_price"], 92);
    // Beef: 2 lb for 9.98 = 11.00/kg.
    assert_eq!(it[2]["base_price"], 1_100);
    // In the tax estimate, the tax printed on that receipt counts (not the state's rate).
    api.call(
        "tax_burden.set_settings",
        json!({"country": "US", "region": "CA"}),
    )
    .unwrap();
    let e = api
        .call("tax_burden.estimate", json!({"year": 2026}))
        .unwrap();
    let line = e["consumption"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["group_key"] == "receipts")
        .expect("receipt lines");
    assert_eq!(line["spent"], 2_991);
    assert_eq!(line["taxes"]["probable"], 109);
}

/// In Brazil (tax in the price) nothing changes: the "tributos" line is not counted and the
/// summed-tax field is ignored.
#[test]
fn included_tax_countries_ignore_the_added_tax() {
    let (_d, api, _, _) = api();
    let p = api
        .call(
            "food.purchase_save",
            json!({"date": "2026-11-14", "store_name": "Mercado Inventado",
                   "items": [{"description": "Arroz 5kg", "total": 2_490}],
                   "tax": 300, "payment": {"mode": "none"}}),
        )
        .unwrap();
    assert_eq!(p["paid"], 2_490);
    assert_eq!(p["tax_added"], 0);
}

/// Open Food Facts categories become product categories: every key in the map exists in the
/// seed, the specific one wins over the general one and a product left in Others is suggested.
#[test]
fn open_data_categories_map_to_product_categories() {
    let (_d, api, _, _) = api();
    let cats = api.call("food.categories", json!({})).unwrap();
    let keys: Vec<&str> = cats
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["seed_key"].as_str())
        .collect();
    for (tag, key) in super::open_data::CATEGORY_MAP {
        assert!(keys.contains(key), "{tag} → {key} is not in the seed");
    }
    let tags = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(
        super::open_data::category_key(&tags(&["en:dairies", "en:cheeses"])),
        Some("laticinios.queijos")
    );
    assert_eq!(
        super::open_data::category_key(&tags(&["en:beverages", "en:waters"])),
        Some("bebidas.agua")
    );
    assert_eq!(super::open_data::category_key(&tags(&["en:unknown"])), None);

    // A product that the description left in Others, with a barcode.
    let p = api
        .call(
            "food.purchase_save",
            json!({"date": "2026-11-10", "store_name": "Mercado Inventado",
                   "items": [{"description": "XPTO 200", "total": 900, "gtin": "7890000000017"}],
                   "payment": {"mode": "none"}}),
        )
        .unwrap();
    let full = api.call("receipts.get", json!({"id": p["id"]})).unwrap();
    let outros = cats
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["seed_key"] == "outros")
        .unwrap()["id"]
        .clone();
    assert_eq!(full["items"][0]["category_id"], outros);
    api.with_session(|s, _| {
        let body = r#"{"status":1,"product":{"product_name":"Queijo Exemplo","categories_tags":["en:dairies","en:cheeses"]}}"#;
        super::open_data::store(
            s.db.conn(),
            &super::open_data::parse("07890000000017", body, "2026-11-11T10:00:00.000Z"),
        )
    })
    .unwrap();
    let after = api.call("receipts.get", json!({"id": p["id"]})).unwrap();
    let queijos = cats
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["seed_key"] == "laticinios.queijos")
        .unwrap()["id"]
        .clone();
    assert_eq!(after["items"][0]["category_id"], queijos);
}
