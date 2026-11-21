use rusqlite::params;

use super::*;
use crate::changes::{self, Author};
use crate::import::testutil::TestEnv;
use crate::transactions::SplitInput;

fn setup() -> (TestEnv, String, String) {
    let mut env = TestEnv::new();
    let conta = env.add_account("Conta Itaú", "checking");
    let outra = env.add_account("Poupança", "savings");
    let mercado = env.category("Mercado");
    let pets = env.category("Ração");
    let t = env.txn(
        &conta,
        "2026-10-01",
        -123456,
        "Supermercado; filial \"centro\"",
        Some(&mercado),
    );
    env.txn(
        &conta,
        "2026-10-05",
        850000,
        "=HYPERLINK(\"http://x\")",
        None,
    );
    env.txn(&outra, "2026-10-06", 1000, "Rendimento", None);
    env.txn(&conta, "2026-11-01", -500, "Tarifa de novembro", None);
    changes::record(env.db.conn_mut(), &Author::user(), "detalhes", |tx| {
        crate::transactions::replace_tags(tx, &t, &["casa".into()])?;
        crate::transactions::replace_splits(
            tx,
            &t,
            -123456,
            &[
                SplitInput {
                    category_id: Some(mercado.clone()),
                    member_id: None,
                    amount: -100000,
                    notes: None,
                },
                SplitInput {
                    category_id: Some(pets.clone()),
                    member_id: None,
                    amount: -23456,
                    notes: None,
                },
            ],
        )?;
        tx.execute(
            "INSERT INTO attachment (id, txn_id, filename, mime, size_bytes, sha256, data, created_at, updated_at)
             VALUES ('anexo1', ?1, 'nota.pdf', 'application/pdf', 4, 'x', ?2, 'a', 'a')",
            params![t, vec![0u8, 159, 146, 150]],
        )?;
        Ok(())
    })
    .unwrap();
    let conn = env.db.conn();
    crate::settings::set(conn, "theme", &"phosphor").unwrap();
    crate::settings::set(conn, "backup.separate_key", &"segredo-hex").unwrap();
    crate::settings::set(conn, "ai.openai.api_key", &"sk-nao-pode-sair").unwrap();
    (env, conta, outra)
}

fn out_dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

#[test]
fn json_export_reads_back_without_secrets() {
    let (env, _conta, _) = setup();
    let dir = out_dir();
    let path = dir.path().join("treasury.json");
    let r = export_json(
        env.db.conn(),
        env.db.schema_version().unwrap(),
        path.to_str().unwrap(),
        &env.paths,
    )
    .unwrap();
    assert!(r.bytes > 0);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(!raw.contains("segredo-hex"));
    assert!(!raw.contains("sk-nao-pode-sair"));
    let v: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v["format"], "treasury-export");
    assert_eq!(v["version"], 1);
    assert_eq!(v["schema_version"], crate::storage::latest_schema_version());
    assert!(v["exported_at"].as_str().unwrap().ends_with('Z'));
    let excluded: Vec<&str> = v["excluded_tables"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect();
    assert!(excluded.contains(&"change_set") && excluded.contains(&"backup_record"));
    let tables = v["tables"].as_object().unwrap();
    assert!(!tables.contains_key("change_item"));
    assert_eq!(tables["txn"].as_array().unwrap().len(), 4);
    assert_eq!(r.counts["txn"], 4);
    assert_eq!(r.counts["txn_split"], 2);
    assert!(r.counts["category"] > 50);
    assert_eq!(tables["household"][0]["name"], "Família");
    let txn = tables["txn"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["amount"] == -123456)
        .unwrap();
    assert_eq!(txn["currency"], "BRL");
    assert_eq!(txn["description"], "Supermercado; filial \"centro\"");
    // Attachment in base64.
    assert_eq!(v["blob_columns"]["attachment"][0], "data");
    let data = tables["attachment"][0]["data"].as_str().unwrap();
    assert_eq!(
        base64::engine::general_purpose::STANDARD
            .decode(data)
            .unwrap(),
        vec![0u8, 159, 146, 150]
    );
    // Settings: theme yes, secrets no.
    let settings: Vec<&str> = tables["setting"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["key"].as_str().unwrap())
        .collect();
    assert!(settings.contains(&"theme"));
    assert!(!settings.iter().any(|k| k.contains("key")));
    // No temporary file left over.
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn secret_settings_filter() {
    assert!(is_secret_setting("backup.separate_key"));
    assert!(is_secret_setting("backup.separate_key.v2"));
    assert!(is_secret_setting("ai.anthropic.api_key"));
    assert!(is_secret_setting("mcp.session_token"));
    assert!(is_secret_setting("Senha_Backup"));
    assert!(!is_secret_setting("theme"));
    assert!(!is_secret_setting("backup"));
    assert!(!is_secret_setting("auto_lock_minutes"));
}

#[test]
fn refuses_bad_destinations() {
    let (env, _, _) = setup();
    let conn = env.db.conn();
    let inside = env.paths.root.join("export.json");
    assert!(export_json(conn, 20, inside.to_str().unwrap(), &env.paths).is_err());
    let backups = env.paths.backups_dir.join("x.json");
    assert!(export_json(conn, 20, backups.to_str().unwrap(), &env.paths).is_err());
    assert!(export_json(conn, 20, "relativo.json", &env.paths).is_err());
    assert!(export_json(conn, 20, "/nao/existe/x.json", &env.paths).is_err());
    let dir = out_dir();
    assert!(export_json(conn, 20, dir.path().to_str().unwrap(), &env.paths).is_err());
}

#[test]
fn transactions_csv_brazilian_format_and_filter() {
    let (env, conta, _) = setup();
    let dir = out_dir();
    let path = dir.path().join("lancamentos.csv");
    let filter = TxnFilter {
        date_from: Some("2026-10-01".into()),
        date_to: Some("2026-10-31".into()),
        account_ids: Some(vec![conta.clone()]),
        ..Default::default()
    };
    let r = export_transactions_csv(
        env.db.conn(),
        Some(&filter),
        path.to_str().unwrap(),
        &env.paths,
    )
    .unwrap();
    assert_eq!(r.counts["txn"], 2);
    let bytes = std::fs::read(&path).unwrap();
    assert!(bytes.starts_with(b"\xEF\xBB\xBF"));
    let text = String::from_utf8(bytes[3..].to_vec()).unwrap();
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(text.as_bytes());
    let headers = rdr.headers().unwrap().clone();
    assert_eq!(&headers[0], "Data");
    assert_eq!(&headers[2], "Descrição");
    assert_eq!(&headers[6], "Valor");
    let rows: Vec<csv::StringRecord> = rdr.records().map(|r| r.unwrap()).collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(&rows[0][0], "01/10/2026", "ascending date order");
    assert_eq!(&rows[0][1], "Conta Itaú");
    assert_eq!(&rows[0][2], "Supermercado; filial \"centro\"");
    assert!(rows[0][4].contains("Mercado"));
    assert_eq!(&rows[0][6], "-1234,56");
    assert_eq!(&rows[0][8], "Despesa");
    assert_eq!(&rows[0][9], "Efetivado");
    assert_eq!(&rows[0][10], "casa");
    assert!(rows[0][14].contains("Ração: -234,56"));
    assert_eq!(&rows[1][2], "'=HYPERLINK(\"http://x\")", "no formula");
    assert_eq!(&rows[1][6], "8500,00");
    assert_eq!(&rows[1][8], "Receita");

    // No filter: all of them.
    let all = dir.path().join("todos.csv");
    let r =
        export_transactions_csv(env.db.conn(), None, all.to_str().unwrap(), &env.paths).unwrap();
    assert_eq!(r.counts["txn"], 4);
}

#[test]
fn decimal_formatting() {
    assert_eq!(decimal_comma(-123456, "BRL"), "-1234,56");
    assert_eq!(decimal_comma(5, "BRL"), "0,05");
    assert_eq!(decimal_comma(1500, "JPY"), "1500");
    assert_eq!(decimal_comma(-1, "KWD"), "-0,001");
}

/// In English, the CSV comes out with a comma as separator, a decimal point, mm/dd/yyyy dates
/// and translated headers, types and status; the JSON's `about` too.
#[test]
fn transactions_csv_follows_the_installation_locale() {
    use crate::i18n::{Locale, set_current};
    let (env, conta, _) = setup();
    let dir = out_dir();
    let path = dir.path().join("transactions.csv");
    let json = dir.path().join("export.json");
    let filter = TxnFilter {
        date_from: Some("2026-10-01".into()),
        date_to: Some("2026-10-31".into()),
        account_ids: Some(vec![conta.clone()]),
        ..Default::default()
    };
    set_current(Locale::En);
    let r = export_transactions_csv(
        env.db.conn(),
        Some(&filter),
        path.to_str().unwrap(),
        &env.paths,
    );
    let j = export_json(env.db.conn(), 20, json.to_str().unwrap(), &env.paths);
    set_current(Locale::PtBr);
    r.unwrap();
    j.unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let text = String::from_utf8(bytes[3..].to_vec()).unwrap();
    let mut rdr = csv::ReaderBuilder::new().from_reader(text.as_bytes());
    let headers = rdr.headers().unwrap().clone();
    assert_eq!(&headers[0], "Date");
    assert_eq!(&headers[6], "Amount");
    let rows: Vec<csv::StringRecord> = rdr.records().map(|r| r.unwrap()).collect();
    assert_eq!(&rows[0][0], "10/01/2026");
    assert_eq!(&rows[0][6], "-1234.56");
    assert_eq!(&rows[0][8], "Expense");
    assert_eq!(&rows[0][9], "Cleared");
    assert!(rows[0][14].contains("Ração: -234.56"));
    let about: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&json).unwrap()).unwrap();
    assert!(
        about["about"]["description"]
            .as_str()
            .unwrap()
            .contains("NOT encrypted")
    );
}
