use base64::Engine as _;
use serde_json::{Value, json};

use crate::api::Api;
use crate::paths::TreasuryPaths;

pub(crate) const NOTE: &str = "NOTA DE NEGOCIAÇÃO\n\
    Nr. nota Folha Data pregão\n\
    123456 1 13/10/2026\n\
    1-BOVESPA C VISTA PETROBRAS PN N2 100 28,50 2.850,00 D\n\
    1-BOVESPA C FRACIONARIO ITAUSA PN N1 7 10,00 70,00 D\n\
    1-BOVESPA V VISTA FII HGLG PAX HGLG11 CI # 10 160,00 1.600,00 C\n\
    Taxa de liquidação 0,95 D\n\
    Emolumentos 0,23 D\n\
    I.R.R.F. s/ operações, base R$ 1.600,00 0,08\n\
    Líquido para 15/10/2026 1.321,26 D\n";

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn setup() -> (tempfile::TempDir, Api, String) {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new(TreasuryPaths::with_root(dir.path().join(".t")).unwrap()).unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Casa"}),
    )
    .unwrap();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Corretora", "kind": "brokerage"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    (dir, api, acc)
}

fn asset(api: &Api, ticker: &str, name: &str, class: &str) -> String {
    api.call(
        "assets.create",
        json!({"ticker": ticker, "name": name, "class": class}),
    )
    .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn preview_resolves_allocates_and_commit_fills_existing_trades() {
    let (_d, api, acc) = setup();
    let petr = asset(&api, "PETR4", "Petrobras", "stock_br");
    let hglg = asset(&api, "HGLG11", "CSHG Logística FII", "fii");
    // The HGLG11 sale already came from the B3 report, with no costs.
    api.call("portfolio.create_trade", json!({"asset_id": hglg, "account_id": acc, "date": "2026-09-01", "side": "buy", "quantity": "10", "price": "150", "cash_movement": false})).unwrap();
    let b3 = api.call("portfolio.create_trade", json!({"asset_id": hglg, "account_id": acc, "date": "2026-10-13", "side": "sell", "quantity": "10", "price": "160", "cash_movement": false})).unwrap();
    let files = json!([{"filename": "nota.txt", "content_base64": b64(NOTE.as_bytes())}]);
    let p = api
        .call("notes.preview", json!({"files": files, "account_id": acc}))
        .unwrap();
    assert!(p["errors"].as_array().unwrap().is_empty(), "{p}");
    let notes = p["notes"].as_array().unwrap();
    assert_eq!(notes[0]["number"], "123456");
    assert_eq!(notes[0]["costs"]["total"], 118);
    assert_eq!(notes[0]["net"], -132126);
    assert_eq!(notes[0]["computed_net"], -132126, "{p}");
    assert!(notes[0]["warnings"].as_array().unwrap().is_empty(), "{p}");
    let trades = p["trades"].as_array().unwrap();
    let by_spec = |s: &str| trades.iter().find(|t| t["spec"] == s).unwrap().clone();
    let pt = by_spec("PETROBRAS PN N2");
    assert_eq!(pt["asset_id"], petr.as_str());
    assert_eq!(pt["status"], "new");
    assert_eq!(
        (pt["fees"].as_i64(), pt["taxes"].as_i64()),
        (Some(75), Some(0))
    );
    let it = by_spec("ITAUSA PN N1");
    assert_eq!(
        it["status"], "unresolved",
        "no ITSA4 registered and no CVM data"
    );
    let hg = by_spec("FII HGLG PAX HGLG11 CI");
    assert_eq!(hg["status"], "fill_costs");
    assert_eq!(hg["existing_trade_id"], b3["id"]);
    assert_eq!(
        (hg["fees"].as_i64(), hg["taxes"].as_i64()),
        (Some(42), Some(8))
    );
    assert_eq!(p["unresolved_specs"], json!(["ITAUSA PN N1"]));

    // The user picks the ITAUSA asset; everything goes in one set of changes.
    let itsa = asset(&api, "ITSA4", "Itaúsa", "stock_br");
    let r = api
        .call(
            "notes.commit",
            json!({"files": files, "account_id": acc, "assets": [{"spec": "ITAUSA PN N1", "asset_id": itsa}]}),
        )
        .unwrap();
    assert_eq!(
        (r["created"].as_u64(), r["updated"].as_u64()),
        (Some(2), Some(1)),
        "{r}"
    );
    let t = api
        .call("portfolio.get_trade", json!({"id": b3["id"]}))
        .unwrap();
    assert_eq!(
        (t["fees"].as_i64(), t["taxes"].as_i64()),
        (Some(42), Some(8))
    );
    assert_eq!(t["note_number"], "123456");

    // Importing again does not duplicate anything.
    let p2 = api
        .call("notes.preview", json!({"files": files, "account_id": acc}))
        .unwrap();
    assert!(
        p2["trades"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|t| t["spec"] != "ITAUSA PN N1")
            .all(|t| t["status"] == "duplicate"),
        "{p2}"
    );

    // Undoing the set brings the costs of the B3 operation back to zero.
    api.call("changes.undo", json!({"id": r["change_set_id"]}))
        .unwrap();
    let t = api
        .call("portfolio.get_trade", json!({"id": b3["id"]}))
        .unwrap();
    assert_eq!(t["fees"], 0);
}

#[test]
fn ticker_in_spec_creates_the_asset_and_options_are_left_out() {
    let (_d, api, acc) = setup();
    let text = "NOTA DE NEGOCIACAO\nData pregão 02/10/2026\nNr. nota 77\n\
        1-BOVESPA C VISTA FII XP LOG XPLG11 CI 5 100,00 500,00 D\n\
        1-BOVESPA C OPCAO DE COMPRA 10/26 PETRJ300 PN 30,00 PETR 100 1,20 120,00 D\n";
    let files = json!([{"filename": "n.txt", "content_base64": b64(text.as_bytes())}]);
    let p = api
        .call("notes.preview", json!({"files": files, "account_id": acc}))
        .unwrap();
    let trades = p["trades"].as_array().unwrap();
    assert_eq!(trades[0]["create_asset"]["ticker"], "XPLG11");
    assert_eq!(trades[0]["create_asset"]["class"], "fii");
    assert_eq!(trades[1]["status"], "unresolved");
    let r = api
        .call("notes.commit", json!({"files": files, "account_id": acc}))
        .unwrap();
    assert_eq!(
        (r["created"].as_u64(), r["assets_created"].as_u64()),
        (Some(1), Some(1))
    );
    let found: Value = api
        .call("assets.find", json!({"ticker": "XPLG11"}))
        .unwrap();
    assert!(!found.is_null(), "{found}");
}

/// A real PDF (Helvetica font, one line of text per instruction), read by the extractor.
pub(crate) fn pdf_with_lines(lines: &[&str]) -> Vec<u8> {
    use lopdf::content::{Content, Operation};
    use lopdf::{Document, Object, Stream, dictionary};
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica", "Encoding" => "WinAnsiEncoding",
    });
    let resources_id = doc.add_object(dictionary! { "Font" => dictionary! { "F1" => font_id } });
    let mut ops = vec![
        Operation::new("BT", vec![]),
        Operation::new("Tf", vec!["F1".into(), 9.into()]),
    ];
    let mut y = 800;
    for l in lines {
        let latin1: Vec<u8> = l.chars().map(|c| c as u32 as u8).collect();
        ops.push(Operation::new(
            "Tm",
            vec![1.into(), 0.into(), 0.into(), 1.into(), 30.into(), y.into()],
        ));
        ops.push(Operation::new(
            "Tj",
            vec![Object::String(latin1, lopdf::StringFormat::Literal)],
        ));
        y -= 14;
    }
    ops.push(Operation::new("ET", vec![]));
    let content = Content { operations: ops };
    let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page", "Parent" => pages_id, "Contents" => content_id,
        "Resources" => resources_id, "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
    });
    doc.objects.insert(
        pages_id,
        Object::Dictionary(
            dictionary! { "Type" => "Pages", "Kids" => vec![page_id.into()], "Count" => 1 },
        ),
    );
    let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog_id);
    let mut out = vec![];
    doc.save_to(&mut out).unwrap();
    out
}

#[test]
fn reads_a_real_pdf() {
    let (_d, api, acc) = setup();
    asset(&api, "PETR4", "Petrobras", "stock_br");
    let pdf = pdf_with_lines(&NOTE.lines().collect::<Vec<_>>());
    let files = json!([{"filename": "nota.pdf", "content_base64": b64(&pdf)}]);
    let p = api
        .call("notes.preview", json!({"files": files, "account_id": acc}))
        .unwrap();
    assert!(p["errors"].as_array().unwrap().is_empty(), "{p}");
    assert_eq!(p["notes"][0]["number"], "123456", "{p}");
    assert_eq!(p["trades"].as_array().unwrap().len(), 3, "{p}");
    assert_eq!(p["notes"][0]["costs"]["irrf"], 8);

    let junk = json!([{"filename": "x.pdf", "content_base64": b64(b"%PDF-1.4 lixo")}]);
    let p = api
        .call("notes.preview", json!({"files": junk, "account_id": acc}))
        .unwrap();
    assert_eq!(p["errors"].as_array().unwrap().len(), 1);
}

/// A real note, outside the repository: `TREASURY_NOTE_PDF=/path/note.pdf cargo test
/// -p treasury-core real_note_from_env -- --ignored --nocapture`. Prints only a summary
/// (counts and the net check), never names or documents.
#[test]
#[ignore = "precisa de TREASURY_NOTE_PDF"]
fn real_note_from_env() {
    let Ok(path) = std::env::var("TREASURY_NOTE_PDF") else {
        return;
    };
    let (_d, api, acc) = setup();
    // Assets as they are in the vault: TREASURY_NOTE_ASSETS="TICKER:Name:class,…".
    for a in std::env::var("TREASURY_NOTE_ASSETS")
        .unwrap_or_default()
        .split(',')
    {
        let p: Vec<&str> = a.split(':').collect();
        if p.len() == 3 {
            asset(&api, p[0], p[1], p[2]);
        }
    }
    // Trading names from a daily COTAHIST: TREASURY_COTAHIST=<zip>.
    if let Ok(z) = std::env::var("TREASURY_COTAHIST") {
        let quotes = crate::research::b3::parse_zip(&std::fs::read(z).unwrap()).unwrap();
        api.with_session(|s, _| super::store_trading_names(s.db.conn(), &quotes))
            .unwrap();
    }
    let group = std::env::var("TREASURY_NOTE_GROUP").as_deref() != Ok("0");
    let bytes = std::fs::read(path).unwrap();
    let files = json!([{"filename": "nota.pdf", "content_base64": b64(&bytes)}]);
    let p = api
        .call(
            "notes.preview",
            json!({"files": files, "account_id": acc, "group": group}),
        )
        .unwrap();
    println!("errors: {}", p["errors"]);
    let notes = p["notes"].as_array().unwrap();
    println!(
        "notes: {}  trades: {}",
        notes.len(),
        p["trades"].as_array().unwrap().len()
    );
    let mut off = 0;
    for n in notes {
        let net = n["net"].as_i64();
        let comp = n["computed_net"].as_i64().unwrap();
        let ok = net == Some(comp);
        if !ok || !n["warnings"].as_array().unwrap().is_empty() {
            off += 1;
            println!(
                "note {} {}: {} trades, net {:?} vs computed {}, costs {}, warnings {}",
                n["number"], n["date"], n["trades"], net, comp, n["costs"]["total"], n["warnings"]
            );
        }
    }
    println!("notes with a difference or warning: {off}");
    println!("specs without an asset: {}", p["unresolved_specs"]);
    let mut specs = std::collections::BTreeMap::new();
    for t in p["trades"].as_array().unwrap() {
        *specs
            .entry(format!("{} -> {}", t["spec"], t["ticker"]))
            .or_insert(0) += 1;
    }
    for (k, v) in specs {
        println!("  {k}: {v}");
    }
    for t in p["trades"].as_array().unwrap() {
        if !t["warning"].is_null() {
            println!("warning {} {}: {}", t["date"], t["ticker"], t["warning"]);
        }
    }
    let r = api
        .call(
            "notes.commit",
            json!({"files": files, "account_id": acc, "group": group}),
        )
        .unwrap();
    println!("saved: {r}");
}

/// 2025 layout (Depositária with a transfer fee), `@`/`ER` marks in the
/// spec, `DRN` BDR, code root in the name and remembered association.
#[test]
fn layout_2025_specs_and_remembered_choices() {
    let (_d, api, acc) = setup();
    asset(&api, "HGLG11", "CSHG Logística FII", "fii");
    asset(&api, "TSLA34", "Tesla BDR", "bdr");
    let gg = asset(&api, "GGRC11", "GGR Covepi Renda FII", "fii");
    let note = "NOTA DE CORRETAGEM\n\
        Nr. Nota Folha Data pregão\n\
        777 1 01/08/2025\n\
        B3 RV LISTADO C VISTA FII HGLG PAX CI ER @ 2 150,00 300,00 D\n\
        B3 RV LISTADO C VISTA TESLA INC DRN @ 10 60,00 600,00 D\n\
        B3 RV LISTADO C VISTA FII GGRCOVEP CI ER 5 10,00 50,00 D\n\
        Taxa de liquidação/CCP 0,23 D\n\
        Emolumentos 0,05 D\n\
        Taxa de Transferência de Ativos 0,16 D\n\
        Líquido para 05/08/2025 950,44 D\n";
    let pdf = pdf_with_lines(&note.lines().collect::<Vec<_>>());
    let files = json!([{"filename": "n.pdf", "content_base64": b64(&pdf)}]);
    let p = api
        .call("notes.preview", json!({"files": files, "account_id": acc}))
        .unwrap();
    assert_eq!(p["notes"][0]["costs"]["total"], 44, "{p}");
    assert!(
        p["notes"][0]["warnings"].as_array().unwrap().is_empty(),
        "{p}"
    );
    let tickers: Vec<Value> = p["trades"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["ticker"].clone())
        .collect();
    assert_eq!(tickers, vec![json!("HGLG11"), json!("TSLA34"), Value::Null]);
    // The choice made at import applies to the following notes (without the marks).
    api.call(
        "notes.commit",
        json!({"files": files, "account_id": acc,
               "assets": [{"spec": "FII GGRCOVEP CI ER", "asset_id": gg}]}),
    )
    .unwrap();
    let again = note.replace("777 1", "778 1").replace("CI ER 5", "CI 5");
    let pdf = pdf_with_lines(&again.lines().collect::<Vec<_>>());
    let files = json!([{"filename": "n2.pdf", "content_base64": b64(&pdf)}]);
    let p = api
        .call("notes.preview", json!({"files": files, "account_id": acc}))
        .unwrap();
    assert_eq!(p["trades"][2]["ticker"], "GGRC11", "{p}");
}

const SPLIT_NOTE: &str = "NOTA DE NEGOCIAÇÃO\n\
    Nr. nota Folha Data pregão\n\
    777 1 14/10/2026\n\
    1-BOVESPA C VISTA FII GGRCOVEP CI ER 10 10,00 100,00 D\n\
    1-BOVESPA C VISTA FII GGRCOVEP CI ER @ 30 10,20 306,00 D\n\
    1-BOVESPA C VISTA FII GGRCOVEP CI 10 10,10 101,00 D\n\
    1-BOVESPA C VISTA BRASIL ON NM 5 20,00 100,00 D\n\
    Taxa de liquidação 0,15 D\n\
    Líquido para 16/10/2026 607,15 D\n";

fn trading_names(api: &Api) {
    let names = json!({
        "GGRC11": ["FII GGRCOVEP", "CI", "12"],
        "BBAS3": ["BRASIL", "ON NM", "02"],
        "TSLA34": ["TESLA INC", "DRN", "34"],
    });
    api.with_session(|s, _| {
        crate::settings::set(s.db.conn(), super::TRADING_NAMES_SETTING, &names)
    })
    .unwrap();
}

#[test]
fn executions_of_the_same_order_are_grouped_with_average_price() {
    let (_d, api, acc) = setup();
    trading_names(&api);
    let files = json!([{"filename": "n.txt", "content_base64": b64(SPLIT_NOTE.as_bytes())}]);
    let p = api
        .call("notes.preview", json!({"files": files, "account_id": acc}))
        .unwrap();
    let trades = p["trades"].as_array().unwrap();
    assert_eq!(trades.len(), 2, "{p}");
    let g = &trades[0];
    assert_eq!(g["executions"], 3);
    assert_eq!(g["quantity"], "50");
    assert_eq!(g["price"], "10.14");
    assert_eq!(g["gross"], 50700);
    assert_eq!(g["spec"], "FII GGRCOVEP CI");
    assert_eq!(g["create_asset"]["ticker"], "GGRC11");
    assert_eq!(g["create_asset"]["class"], "fii");
    assert_eq!(trades[1]["create_asset"]["ticker"], "BBAS3");
    assert_eq!(p["notes"][0]["computed_net"], -60715, "{p}");
    // Not grouped: one line per execution.
    let p = api
        .call(
            "notes.preview",
            json!({"files": files, "account_id": acc, "group": false}),
        )
        .unwrap();
    assert_eq!(p["trades"].as_array().unwrap().len(), 4);
    // Saved separately before: the grouped one recognizes the sum and does not duplicate.
    let r = api
        .call(
            "notes.commit",
            json!({"files": files, "account_id": acc, "group": false}),
        )
        .unwrap();
    assert_eq!(r["created"], 4, "{r}");
    let p = api
        .call("notes.preview", json!({"files": files, "account_id": acc}))
        .unwrap();
    assert_eq!(p["trades"][0]["status"], "duplicate", "{p}");
    assert_eq!(p["trades"][1]["status"], "duplicate", "{p}");
}

#[test]
fn any_line_can_be_corrected_before_commit() {
    let (_d, api, acc) = setup();
    trading_names(&api);
    let bbas = asset(&api, "BBSE3", "BB Seguridade", "stock_br");
    let files = json!([{"filename": "n.txt", "content_base64": b64(SPLIT_NOTE.as_bytes())}]);
    // The user corrects BRASIL ON NM to another asset and gives a new code for the FII.
    let assets = json!([
        {"spec": "BRASIL ON NM", "asset_id": bbas},
        {"spec": "FII GGRCOVEP CI", "ticker": "ggrc11", "class": "fii"},
    ]);
    let p = api
        .call(
            "notes.preview",
            json!({"files": files, "account_id": acc, "assets": assets}),
        )
        .unwrap();
    let t = p["trades"].as_array().unwrap();
    assert_eq!(t[1]["asset_id"], bbas.as_str());
    assert_eq!(t[1]["chosen"], true);
    assert_eq!(t[0]["create_asset"]["ticker"], "GGRC11");
    let r = api
        .call(
            "notes.commit",
            json!({"files": files, "account_id": acc, "assets": assets}),
        )
        .unwrap();
    assert_eq!(
        (r["created"].as_i64(), r["assets_created"].as_i64()),
        (Some(2), Some(1))
    );
    // The next note already comes with the corrections.
    let p = api
        .call("notes.preview", json!({"files": files, "account_id": acc}))
        .unwrap();
    assert_eq!(p["trades"][1]["asset_id"], bbas.as_str(), "{p}");
    assert_eq!(p["trades"][1]["status"], "duplicate");
    let bad = api.call(
        "notes.preview",
        json!({"files": files, "account_id": acc, "assets": [{"spec": "X"}]}),
    );
    assert!(bad.is_err());
}

#[test]
fn sale_bigger_than_position_is_flagged_but_imported() {
    let (_d, api, acc) = setup();
    asset(&api, "PETR4", "Petrobras", "stock_br");
    let text = "NOTA DE NEGOCIAÇÃO\n\
        Nr. nota Folha Data pregão\n\
        900 1 13/10/2026\n\
        1-BOVESPA V VISTA PETROBRAS PN N2 100 30,00 3.000,00 C\n\
        1-BOVESPA C VISTA PETROBRAS PN N2 40 29,00 1.160,00 D\n\
        Líquido para 15/10/2026 1.840,00 C\n";
    let files = json!([{"filename": "n.txt", "content_base64": b64(text.as_bytes())}]);
    let p = api
        .call("notes.preview", json!({"files": files, "account_id": acc}))
        .unwrap();
    let trades = p["trades"].as_array().unwrap();
    let sell = trades.iter().find(|t| t["side"] == "sell").unwrap();
    // The same-day purchase counts before the sale: 40 in the portfolio.
    assert!(
        sell["warning"].as_str().unwrap().contains("posição de 40"),
        "{sell}"
    );
    assert!(trades.iter().find(|t| t["side"] == "buy").unwrap()["warning"].is_null());
    let r = api
        .call("notes.commit", json!({"files": files, "account_id": acc}))
        .unwrap();
    assert_eq!(r["created"], 2, "{r}");
}
