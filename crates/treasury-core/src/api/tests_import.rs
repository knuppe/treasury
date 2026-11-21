//! Import, rules and export through the dispatcher (`Api::call`). Accounts are created
//! with direct SQL because the account CRUD commands belong to another module.

use base64::Engine as _;
use serde_json::{Value, json};

use super::*;

const NUBANK_CONTA: &[u8] = include_bytes!("../import/fixtures/nubank_conta.csv");
const CARTAO_OFX: &[u8] = include_bytes!("../import/fixtures/cartao_credito.ofx");
const ITAU_OFX: &[u8] = include_bytes!("../import/fixtures/itau_sgml_latin1.ofx");

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn setup() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new_unlocked_process(paths).unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Família", "members": ["Ana", "Bruno"]}),
    )
    .unwrap();
    (dir, api)
}

fn sql_account(api: &Api, name: &str, kind: &str) -> String {
    let id = crate::ids::new_id();
    api.with_session(|s, _| {
        s.db.conn().execute(
            "INSERT INTO account (id, name, kind, currency, opening_date, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'BRL', '2026-01-01', 'x', 'x')",
            rusqlite::params![id, name, kind],
        )?;
        if kind == "credit_card" {
            s.db.conn().execute(
                "INSERT INTO credit_card (account_id, closing_day, due_day, created_at, updated_at)
                 VALUES (?1, 25, 5, 'x', 'x')",
                [&id],
            )?;
        }
        Ok(())
    })
    .unwrap();
    id
}

fn category(api: &Api, name: &str) -> String {
    api.with_session(|s, _| {
        Ok(s.db.conn().query_row(
            "SELECT id FROM category WHERE name = ?1 ORDER BY parent_id IS NOT NULL LIMIT 1",
            [name],
            |r| r.get::<_, String>(0),
        )?)
    })
    .unwrap()
}

fn count(api: &Api, sql: &str) -> i64 {
    api.with_session(|s, _| Ok(s.db.conn().query_row(sql, [], |r| r.get::<_, i64>(0))?))
        .unwrap()
}

#[test]
fn import_flow_detect_preview_commit_undo_redo() {
    let (_d, api) = setup();
    let conta = sql_account(&api, "Nubank", "checking");

    let presets = api.call("import.list_presets", Value::Null).unwrap();
    let ids: Vec<&str> = presets
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    for want in [
        "nubank_conta",
        "nubank_cartao",
        "inter",
        "itau",
        "c6",
        "generico",
    ] {
        assert!(ids.contains(&want), "preset {want}");
    }

    let d = api
        .call(
            "import.detect",
            json!({"filename": "NU.csv", "content_base64": b64(NUBANK_CONTA)}),
        )
        .unwrap();
    assert_eq!(d["format"], "csv");
    assert_eq!(d["delimiter"], ",");
    assert_eq!(d["suggested_preset_id"], "nubank_conta");
    assert_eq!(d["headers"][3], "Descrição");

    let p = api
        .call(
            "import.preview",
            json!({"account_id": conta, "filename": "NU.csv", "content_base64": b64(NUBANK_CONTA)}),
        )
        .unwrap();
    assert_eq!(p["counts"]["new"], 5);
    assert_eq!(p["already_imported"], false);
    assert_eq!(p["rows"][3]["kind"], "card_payment");

    let r = api
        .call("import.commit", json!({"token": p["token"]}))
        .unwrap();
    assert_eq!(r["inserted"], 5);
    let cs = r["change_set_id"].as_str().unwrap().to_string();
    assert_eq!(count(&api, "SELECT count(*) FROM txn"), 5);

    let batches = api.call("import.list_batches", json!({})).unwrap();
    assert_eq!(batches[0]["change_set_id"], cs.as_str());
    assert_eq!(batches[0]["format"], "csv");
    let by_account = api
        .call("import.list_batches", json!({"account_id": conta}))
        .unwrap();
    assert_eq!(by_account.as_array().unwrap().len(), 1);

    let hist = api
        .call("changes.list", json!({"author_kind": "import"}))
        .unwrap();
    assert_eq!(hist[0]["reason"], "Importação de NU.csv");

    let again = api
        .call(
            "import.preview",
            json!({"account_id": conta, "filename": "NU.csv", "content_base64": b64(NUBANK_CONTA)}),
        )
        .unwrap();
    assert_eq!(again["already_imported"], true);
    assert_eq!(again["counts"]["duplicate"], 5);

    api.call("changes.undo", json!({"id": cs})).unwrap();
    assert_eq!(count(&api, "SELECT count(*) FROM txn"), 0);
    assert_eq!(count(&api, "SELECT count(*) FROM import_batch"), 0);
    assert_eq!(
        api.call("import.list_batches", json!({}))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        0
    );
    api.call("changes.redo", json!({"id": cs})).unwrap();
    assert_eq!(count(&api, "SELECT count(*) FROM txn"), 5);

    // Invalid token and bad arguments.
    assert!(matches!(
        api.call("import.commit", json!({"token": "nao-existe"})),
        Err(Error::NotFound(_))
    ));
    assert!(matches!(
        api.call("import.preview", json!({"account_id": conta})),
        Err(Error::Invalid(_))
    ));
    assert!(matches!(
        api.call("import.nada", Value::Null),
        Err(Error::UnknownCommand(_))
    ));
}

#[test]
fn rules_crud_test_apply_and_suggest() {
    let (_d, api) = setup();
    let card = sql_account(&api, "Cartão", "credit_card");
    let apps = category(&api, "Aplicativos");
    let mercado = category(&api, "Mercado");

    let bad = api.call(
        "rules.create",
        json!({"name": "x", "conditions": [{"field": "description", "op": "regex", "value": "(uber"}],
               "actions": {"category_id": apps}}),
    );
    assert_eq!(bad.unwrap_err().code(), "invalid");

    let rule = api
        .call(
            "rules.create",
            json!({"name": "Uber", "priority": 10,
                   "conditions": [{"field": "description", "op": "contains", "value": "uber"}],
                   "actions": {"category_id": apps, "tags": ["transporte"]}}),
        )
        .unwrap();
    let rule_id = rule["id"].as_str().unwrap().to_string();
    assert_eq!(rule["match_all"], true);
    assert_eq!(rule["hits"], 0);
    // Besides the vault's ready-made rules (iFood, Netflix…), the new one.
    assert_eq!(
        api.call("rules.list", Value::Null)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["priority"].as_i64() < Some(900))
            .count(),
        1
    );

    // Import uses the rule.
    let p = api
        .call(
            "import.preview",
            json!({"account_id": card, "filename": "fatura.ofx", "content_base64": b64(CARTAO_OFX)}),
        )
        .unwrap();
    let uber = p["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["description"].as_str().unwrap().contains("UBER"))
        .unwrap()
        .clone();
    assert_eq!(uber["suggested_category_id"], apps.as_str());
    assert_eq!(uber["rule_id"], rule_id.as_str());
    api.call("import.commit", json!({"token": p["token"]}))
        .unwrap();
    assert_eq!(
        api.call("rules.get", json!({"id": rule_id})).unwrap()["hits"],
        1
    );

    // Testing a new rule against what already exists, without writing.
    let tested = api
        .call(
            "rules.test",
            json!({"rule": {"name": "mercado", "conditions": [{"field": "description", "op": "contains", "value": "MERCADO"}],
                            "actions": {"category_id": mercado}}}),
        )
        .unwrap();
    assert_eq!(tested.as_array().unwrap().len(), 1);
    assert_eq!(tested[0]["account_name"], "Cartão");

    let m = api
        .call(
            "rules.create",
            json!({"name": "Mercado", "conditions": [{"field": "description", "op": "contains", "value": "mercado"},
                                                      {"field": "amount", "op": "lte", "value": -1000}],
                   "actions": {"category_id": mercado}}),
        )
        .unwrap();
    let dry = api
        .call(
            "rules.apply",
            json!({"rule_ids": [m["id"]], "dry_run": true}),
        )
        .unwrap();
    assert_eq!(dry["updated"], 1);
    assert!(dry["change_set_id"].is_null());
    assert_eq!(dry["sample"][0]["category_id"], mercado.as_str());
    let real = api
        .call("rules.apply", json!({"rule_ids": [m["id"]]}))
        .unwrap();
    assert_eq!(real["updated"], 1);
    let cs = real["change_set_id"].as_str().unwrap().to_string();
    // Uber (the test's rule), IOF (ready-made rule) and now the grocery store.
    assert_eq!(
        count(
            &api,
            "SELECT count(*) FROM txn WHERE category_id IS NOT NULL"
        ),
        3
    );
    api.call("changes.undo", json!({"id": cs})).unwrap();
    assert_eq!(
        count(
            &api,
            "SELECT count(*) FROM txn WHERE category_id IS NOT NULL"
        ),
        2
    );

    // Suggestion from a transaction.
    let txn_id: String = api
        .with_session(|s, _| {
            Ok(s.db.conn().query_row(
                "SELECT id FROM txn WHERE description LIKE 'UBER%'",
                [],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    let s = api
        .call("rules.suggest_from_txn", json!({"txn_id": txn_id}))
        .unwrap();
    assert_eq!(s["conditions"][0]["op"], "contains");
    assert_eq!(s["conditions"][0]["value"], "uber");
    assert_eq!(s["actions"]["category_id"], apps.as_str());

    let upd = api
        .call(
            "rules.update",
            json!({"id": rule_id, "name": "Uber e 99", "match_all": false,
                   "conditions": [{"field": "description", "op": "contains", "value": "uber"},
                                  {"field": "description", "op": "contains", "value": "99app"}],
                   "actions": {"category_id": apps}}),
        )
        .unwrap();
    assert_eq!(upd["name"], "Uber e 99");
    assert_eq!(upd["match_all"], false);
    assert_eq!(upd["priority"], 10, "priority kept");
    assert_eq!(upd["hits"], 1, "hit counts kept");
    api.call("rules.delete", json!({"id": rule_id})).unwrap();
    assert!(matches!(
        api.call("rules.get", json!({"id": rule_id})),
        Err(Error::NotFound(_))
    ));
}

#[test]
fn export_via_api() {
    let (dir, api) = setup();
    let conta = sql_account(&api, "Itaú", "checking");
    let p = api
        .call(
            "import.preview",
            json!({"account_id": conta, "filename": "itau.ofx", "content_base64": b64(ITAU_OFX)}),
        )
        .unwrap();
    api.call("import.commit", json!({"token": p["token"]}))
        .unwrap();
    let out = dir.path().join("saida");
    std::fs::create_dir(&out).unwrap();

    let path = out.join("tudo.json");
    let r = api
        .call("export.json", json!({"path": path.to_str().unwrap()}))
        .unwrap();
    assert_eq!(r["counts"]["txn"], 6);
    assert_eq!(r["counts"]["import_batch"], 1);
    let v: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(v["format"], "treasury-export");
    let info = api.call("app.info", Value::Null).unwrap();
    assert_eq!(v["schema_version"], info["schema_version"]);

    let csv = out.join("lancamentos.csv");
    let r = api
        .call(
            "export.transactions_csv",
            json!({"path": csv.to_str().unwrap(), "filter": {"date_from": "2026-10-05", "account_ids": [conta]}}),
        )
        .unwrap();
    assert_eq!(r["counts"]["txn"], 4);
    let text = std::fs::read_to_string(&csv).unwrap();
    assert!(text.contains("05/10/2026;Itaú;SISPAG SALÁRIO EMPRESA EXEMPLO;"));
    assert!(text.contains(";8500,00;BRL;Receita;"));

    let inside = api.paths().root.join("x.json");
    assert!(
        api.call("export.json", json!({"path": inside.to_str().unwrap()}))
            .is_err()
    );
}

#[test]
fn read_only_mcp_permissions() {
    let (_d, api) = setup();
    let conta = sql_account(&api, "Conta", "checking");
    let ro = Actor::Mcp {
        client: "claude-desktop".into(),
        read_only: true,
    };
    assert!(api.call_as(&ro, "import.list_presets", Value::Null).is_ok());
    assert!(api.call_as(&ro, "import.list_batches", json!({})).is_ok());
    assert!(
        api.call_as(
            &ro,
            "import.detect",
            json!({"filename": "NU.csv", "content_base64": b64(NUBANK_CONTA)})
        )
        .is_ok()
    );
    assert!(api.call_as(&ro, "rules.list", Value::Null).is_ok());
    for (cmd, args) in [
        (
            "import.preview",
            json!({"account_id": conta, "filename": "NU.csv", "content_base64": b64(NUBANK_CONTA)}),
        ),
        ("import.commit", json!({"token": "x"})),
        ("rules.apply", json!({})),
        ("rules.create", json!({})),
        ("export.json", json!({"path": "/tmp/x.json"})),
        ("export.transactions_csv", json!({"path": "/tmp/x.csv"})),
    ] {
        assert!(
            matches!(api.call_as(&ro, cmd, args), Err(Error::Forbidden(_))),
            "{cmd} must require write access"
        );
    }
    // An MCP client with write access imports as "import" (batch author), not as AI.
    let writer = Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    };
    let p = api
        .call_as(
            &writer,
            "import.preview",
            json!({"account_id": conta, "filename": "NU.csv", "content_base64": b64(NUBANK_CONTA)}),
        )
        .unwrap();
    let r = api
        .call_as(&writer, "import.commit", json!({"token": p["token"]}))
        .unwrap();
    // Importing through the AI is sensitive: becomes a proposal and only counts after approval.
    assert_eq!(r["status"], "pending_approval");
    api.call("ai.approve", json!({"ids": [r["proposal_id"]]}))
        .unwrap();
    let hist = api.call("changes.list", json!({})).unwrap();
    assert_eq!(hist[0]["author"]["kind"], "import");
    assert_eq!(hist[0]["author"]["detail"], "NU.csv — agente");
}

#[test]
fn large_import_creates_backup_via_api() {
    let (_d, api) = setup();
    let conta = sql_account(&api, "Conta", "checking");
    let mut csv = String::from("Data,Valor,Identificador,Descrição\n");
    for i in 0..(crate::import::BACKUP_THRESHOLD + 1) {
        csv.push_str(&format!(
            "{:02}/09/2026,-{}.00,id-{i},Compra {i}\n",
            i % 28 + 1,
            i + 1
        ));
    }
    let p = api
        .call(
            "import.preview",
            json!({"account_id": conta, "filename": "grande.csv", "content_base64": b64(csv.as_bytes())}),
        )
        .unwrap();
    let r = api
        .call("import.commit", json!({"token": p["token"]}))
        .unwrap();
    assert!(r["backup_id"].is_string());
    let backups = api.call("backup.list", Value::Null).unwrap();
    assert!(
        backups
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b["kind"] == "pre_import" && b["id"] == r["backup_id"])
    );
}

// ---------------------------------------------------------------------------
// Caixa statement by period (docs/brazil/caixa-statement-by-period.md): all made up.

const CAIXA_OCR: &str = include_str!("../import/fixtures/caixa-extrato-periodo.ocr.txt");

/// PDF with a text layer (one line of read text per PDF line).
fn text_pdf(lines: &[&str]) -> Vec<u8> {
    use lopdf::content::{Content, Operation};
    use lopdf::{Document, Object, Stream, dictionary};
    let mut doc = Document::with_version("1.3");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Courier", "Encoding" => "WinAnsiEncoding",
    });
    let resources_id = doc.add_object(dictionary! { "Font" => dictionary! { "F1" => font_id } });
    let mut ops = vec![
        Operation::new("BT", vec![]),
        Operation::new("Tf", vec!["F1".into(), 7.into()]),
    ];
    let mut y = 820;
    for l in lines {
        let latin1: Vec<u8> = l.chars().map(|c| c as u32 as u8).collect();
        ops.push(Operation::new(
            "Tm",
            vec![1.into(), 0.into(), 0.into(), 1.into(), 20.into(), y.into()],
        ));
        ops.push(Operation::new(
            "Tj",
            vec![Object::String(latin1, lopdf::StringFormat::Literal)],
        ));
        y -= 12;
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

/// PDF like the Internet Banking one: `pdfmake`, A4, the page is a single JPEG image.
fn image_pdf(jpeg: &[u8], w: i64, h: i64) -> Vec<u8> {
    use lopdf::content::{Content, Operation};
    use lopdf::{Document, Object, Stream, dictionary};
    let mut doc = Document::with_version("1.3");
    let pages_id = doc.new_object_id();
    let img = Stream::new(
        dictionary! {
            "Type" => "XObject", "Subtype" => "Image", "Width" => w, "Height" => h,
            "ColorSpace" => "DeviceRGB", "BitsPerComponent" => 8, "Filter" => "DCTDecode",
        },
        jpeg.to_vec(),
    );
    let img_id = doc.add_object(img);
    let resources_id = doc.add_object(dictionary! { "XObject" => dictionary! { "I1" => img_id } });
    let content = Content {
        operations: vec![
            Operation::new("q", vec![]),
            Operation::new(
                "cm",
                vec![
                    595.into(),
                    0.into(),
                    0.into(),
                    842.into(),
                    0.into(),
                    0.into(),
                ],
            ),
            Operation::new("Do", vec!["I1".into()]),
            Operation::new("Q", vec![]),
        ],
    };
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
    let info_id = doc.add_object(dictionary! {
        "Creator" => Object::string_literal("pdfmake"), "Producer" => Object::string_literal("pdfmake"),
    });
    let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog_id);
    doc.trailer.set("Info", info_id);
    let mut out = vec![];
    doc.save_to(&mut out).unwrap();
    out
}

#[test]
fn caixa_statement_by_period_imports_without_ai() {
    let (_d, api) = setup();
    let conta = sql_account(&api, "Caixa", "checking");
    let lines: Vec<&str> = CAIXA_OCR.lines().collect();
    let pdf = text_pdf(&lines);
    let s = api
        .call(
            "import.sniff",
            json!({"filename": "Comprovante_2025-03-15_093000.pdf", "content_base64": b64(&pdf)}),
        )
        .unwrap();
    assert_eq!(s["kind"], "statement", "{s}");
    assert_eq!(s["layout"]["id"], "bank.caixa.extrato-periodo.v1");
    let d = api
        .call(
            "import.detect",
            json!({"filename": "extrato.pdf", "content_base64": b64(&pdf)}),
        )
        .unwrap();
    assert_eq!(d["row_count"], 4);
    assert_eq!(d["layout"]["subject"], "bank_statement");
    let p = api
        .call(
            "import.preview",
            json!({"account_id": conta, "filename": "extrato.pdf", "content_base64": b64(&pdf)}),
        )
        .unwrap();
    assert_eq!(p["counts"]["new"], 4, "{p}");
    let rows = p["rows"].as_array().unwrap();
    let amounts: Vec<i64> = rows.iter().map(|r| r["amount"].as_i64().unwrap()).collect();
    assert_eq!(amounts, vec![200_000, -12_050, -7_990, -320_000]);
    assert_eq!(rows[0]["payee"], "Fulano Beltrano Si");
    assert!(
        p["warnings"].as_array().unwrap().is_empty(),
        "{}",
        p["warnings"]
    );
    assert!(!p.to_string().contains("123.456"));
    let r = api
        .call("import.commit", json!({"token": p["token"]}))
        .unwrap();
    assert_eq!(r["inserted"], 4);
    // Again (or in an overlapping period): nothing is duplicated.
    let again = api
        .call(
            "import.preview",
            json!({"account_id": conta, "filename": "outro-nome.pdf", "content_base64": b64(&text_pdf(&lines[..13]))}),
        )
        .unwrap();
    assert_eq!(again["counts"]["new"], 0, "{again}");
    // One wrong digit: the day that does not add up becomes a warning.
    let wrong = CAIXA_OCR.replace("120,50 D", "126,50 D");
    let wrong_lines: Vec<&str> = wrong.lines().collect();
    let conta2 = sql_account(&api, "Caixa 2", "checking");
    let w = api
        .call(
            "import.preview",
            json!({"account_id": conta2, "filename": "e.pdf", "content_base64": b64(&text_pdf(&wrong_lines))}),
        )
        .unwrap();
    let warns = w["warnings"].to_string();
    assert!(warns.contains("não fecha"), "{warns}");
    // On a card account, it is still only a statement.
    let cartao = sql_account(&api, "Cartão", "credit_card");
    assert!(
        api.call(
            "import.preview",
            json!({"account_id": cartao, "filename": "e.pdf", "content_base64": b64(&pdf)}),
        )
        .is_err()
    );
}

#[cfg(feature = "ocr-paddle")]
#[test]
fn image_only_pdf_goes_to_the_local_ocr_not_to_brokerage_notes() {
    let mut jpeg = vec![];
    image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
        64,
        96,
        image::Rgb([255, 255, 255]),
    ))
    .write_to(
        &mut std::io::Cursor::new(&mut jpeg),
        image::ImageFormat::Jpeg,
    )
    .unwrap();
    let pdf = image_pdf(&jpeg, 64, 96);
    // The image comes out of the PDF byte for byte.
    let pages = crate::pdf::page_jpegs(&pdf).unwrap();
    assert_eq!(pages.jpegs, vec![jpeg]);
    assert_eq!(crate::pdf::text(&pdf, None).unwrap().trim(), "");
    let (_d, api) = setup();
    let s = api
        .call(
            "import.sniff",
            json!({"filename": "Comprovante_2025-03-15_093000.pdf", "content_base64": b64(&pdf)}),
        )
        .unwrap();
    // Blank page: nothing readable, with a clear warning (neither a brokerage note nor an error).
    assert_eq!(s["kind"], "unknown");
    assert_ne!(s["kind"], "brokerage_note");
    assert!(s["note"].as_str().is_some_and(|n| !n.is_empty()));
}

/// End to end with the real OCR: a synthetic image of the statement (outside the
/// repository) wrapped in a PDF like the Internet Banking one. `TREASURY_CAIXA_SAMPLE=<jpeg>
/// cargo test --release -p treasury-core --lib caixa_sample -- --ignored --nocapture`.
#[cfg(feature = "ocr-paddle")]
#[test]
#[ignore]
fn caixa_sample_through_the_local_ocr() {
    let Ok(path) = std::env::var("TREASURY_CAIXA_SAMPLE") else {
        return;
    };
    let jpeg = std::fs::read(path).unwrap();
    let pdf = image_pdf(&jpeg, 1957, 3230);
    let (text, conf) = crate::import::statement_text(&pdf).unwrap();
    if std::env::var("TREASURY_SAMPLES_DEBUG").is_ok() {
        eprintln!("{text}");
    }
    assert!(conf.is_some());
    assert!(crate::import::caixa::looks_like(&text));
    let st = crate::import::caixa::parse(&text);
    let got: Vec<i64> = st.rows.iter().map(|r| r.amount).collect();
    eprintln!(
        "transactions {}/4, opening {:?}, closing {:?}, total {:?}, days with problems {}",
        got.len(),
        st.opening,
        st.closing,
        st.total_ok,
        st.problems.len()
    );
    assert_eq!(got, vec![200_000, -12_050, -7_990, -320_000]);
    assert_eq!(st.total_ok, Some(true));
    let (_d, api) = setup();
    let s = api
        .call(
            "import.sniff",
            json!({"filename": "Comprovante_2025-03-15_093000.pdf", "content_base64": b64(&pdf)}),
        )
        .unwrap();
    assert_eq!(s["kind"], "statement", "{s}");
}

/// Statement given in the batch import (feedback 01a199fa): per line beats the batch's, which
/// beats the closing day; cards only.
#[test]
fn rows_import_puts_purchases_in_the_informed_statement() {
    let (_d, api) = setup();
    let card = sql_account(&api, "Cartão", "credit_card");
    let r = api
        .call(
            "import.rows",
            json!({"account_id": card, "source": "fatura do cartão na página",
            "statement_due_date": "2026-11-05",
            "rows": [
                {"date": "2026-10-20", "amount": -5_000, "description": "LOJA A"},
                {"date": "2026-10-24", "amount": -7_000, "description": "LOJA B",
                 "statement_due_date": "2026-12-05"},
                {"date": "2026-10-27", "amount": -9_000, "description": "LOJA C"}
            ]}),
        )
        .unwrap();
    assert_eq!(r["status"], "applied", "{r}");
    let months: Vec<(String, String)> = api
        .with_session(|s, _| {
            let mut st = s.db.conn().prepare(
                "SELECT t.description, s.reference_month FROM txn t
                 JOIN card_statement s ON s.id = t.statement_id ORDER BY t.description",
            )?;
            let v = st
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(v)
        })
        .unwrap();
    assert_eq!(
        months,
        vec![
            ("LOJA A".to_string(), "2026-11".to_string()),
            ("LOJA B".to_string(), "2026-12".to_string()),
            // After the closing (day 25) it would go to December; the batch says November.
            ("LOJA C".to_string(), "2026-11".to_string()),
        ]
    );
    let conta = sql_account(&api, "Conta", "checking");
    let e = api
        .call(
            "import.rows",
            json!({"account_id": conta, "source": "extrato", "statement_due_date": "2026-11-05",
                   "rows": [{"date": "2026-10-20", "amount": -5_000, "description": "X"}]}),
        )
        .unwrap_err();
    assert_eq!(e.code(), "invalid");
    let e = api
        .call(
            "import.rows",
            json!({"account_id": card, "source": "fatura", "statement_due_date": "05/11/2026",
                   "rows": [{"date": "2026-10-20", "amount": -5_000, "description": "Y"}]}),
        )
        .unwrap_err();
    assert_eq!(e.code(), "invalid");
}
