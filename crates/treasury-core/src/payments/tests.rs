//! Tests for boletos, Pix and receipts: all codes, names, keys and amounts are
//! made up; the check digits are computed here.

use chrono::NaiveDate;
use serde_json::json;

use super::boleto::{self, BillKind, mod10, mod11_bank, mod11_collection};
use super::proof::{Direction, ProofKind};
use super::*;
use crate::paths::TreasuryPaths;

fn d(s: &str) -> NaiveDate {
    parse_date(s).unwrap()
}

fn digits(s: &str) -> Vec<u32> {
    s.chars().map(|c| c.to_digit(10).unwrap()).collect()
}

/// Made-up bank boleto: bank, factor, amount and free field, with the overall DV computed.
fn bank_barcode(bank: &str, factor: u32, cents: u64, free: &str) -> String {
    let body = format!("{bank}9{factor:04}{cents:010}{free}");
    let dv = mod11_bank(&digits(&body));
    format!("{}{dv}{}", &body[..4], &body[4..])
}

/// Made-up collection bill (segment, amount identifier 6 = modulo 10).
fn collection_barcode(segment: u32, cents: u64, rest: &str) -> String {
    let body = format!("8{segment}6{cents:011}{rest}");
    let dv = mod10(&digits(&body));
    format!("{}{dv}{}", &body[..3], &body[3..])
}

const FREE: &str = "1234567890123456789012345";

#[test]
fn check_digits_follow_febraban() {
    // Classic examples from the manuals: modulo 10 of 01230067896 = 3; modulo 11 of 0019373700000001000500940144816060680935031 = 3.
    assert_eq!(mod10(&digits("01230067896")), 3);
    assert_eq!(
        mod11_bank(&digits("0019373700000001000500940144816060680935031")),
        3
    );
    // Collection: remainder 0 or 1 gives 0.
    assert_eq!(mod11_collection(&digits("0")), 0);
}

#[test]
fn due_factor_resolves_the_2025_restart() {
    assert_eq!(
        boleto::due_from_factor(9999, d("2025-01-10")),
        Some(d("2025-02-21"))
    );
    assert_eq!(
        boleto::due_from_factor(1000, d("2025-03-01")),
        Some(d("2025-02-22"))
    );
    assert_eq!(
        boleto::due_from_factor(1000, d("2000-01-01")),
        Some(d("2000-07-03"))
    );
    // Factor of a November 2026 bill: 1000 + days since 22/02/2025.
    let f = 1000 + (d("2026-11-10") - d("2025-02-22")).num_days() as u32;
    assert_eq!(
        boleto::due_from_factor(f, d("2026-10-23")),
        Some(d("2026-11-10"))
    );
    assert_eq!(boleto::due_from_factor(0, d("2026-10-23")), None);
}

#[test]
fn bank_boleto_from_barcode_and_line() {
    let today = d("2026-10-23");
    let f = 1000 + (d("2026-11-10") - d("2025-02-22")).num_days() as u32;
    let code = bank_barcode("341", f, 12_345, FREE);
    let b = boleto::from_barcode(&code, today).unwrap();
    assert_eq!(b.kind, BillKind::Bank);
    assert_eq!(b.amount, Some(12_345));
    assert_eq!(b.due_date.as_deref(), Some("2026-11-10"));
    assert_eq!(b.bank_name.as_deref(), Some("Itaú"));
    // The digitable line leads back to the same barcode.
    assert_eq!(boleto::from_line(&b.line, today).unwrap().barcode, code);
    assert_eq!(b.line.chars().filter(char::is_ascii_digit).count(), 47);
    // One swapped digit does not pass.
    let mut bad: Vec<char> = code.chars().collect();
    bad[20] = if bad[20] == '9' { '8' } else { '9' };
    assert!(boleto::from_barcode(&bad.iter().collect::<String>(), today).is_none());
    let mut bad_line: Vec<char> = b.line.chars().collect();
    let i = bad_line.iter().position(|c| c.is_ascii_digit()).unwrap() + 2;
    bad_line[i] = if bad_line[i] == '7' { '6' } else { '7' };
    assert!(boleto::from_line(&bad_line.iter().collect::<String>(), today).is_none());
}

#[test]
fn collection_bill_by_segment() {
    let today = d("2026-10-23");
    let code = collection_barcode(3, 18_790, "00001234567890123456789012345");
    assert_eq!(code.len(), 44);
    let b = boleto::from_barcode(&code, today).unwrap();
    assert_eq!(b.kind, BillKind::Collection);
    assert_eq!(b.segment, Some(3));
    assert_eq!(b.amount, Some(18_790));
    assert_eq!(boleto::category_seed(&b), "categorias-energia");
    assert_eq!(b.line.chars().filter(char::is_ascii_digit).count(), 48);
    assert_eq!(boleto::from_line(&b.line, today).unwrap().barcode, code);
}

#[test]
fn finds_the_line_in_ocr_text_with_confused_letters_and_a_wrap() {
    let today = d("2026-10-23");
    let f = 1000 + (d("2026-11-10") - d("2025-02-22")).num_days() as u32;
    let b = boleto::from_barcode(&bank_barcode("237", f, 45_000, FREE), today).unwrap();
    // OCR swaps a 0 for an O and breaks the digitable line in two.
    let line = b.line.replacen('0', "O", 1);
    let (a, z) = line.split_at(line.find(' ').unwrap() + 13);
    let rows: Vec<String> = [
        "Banco Exemplo",
        "Beneficiário: Escola Exemplo de Testes Ltda  CNPJ 00.000.000/0001-00",
        a,
        z,
        "Vencimento 10/11/2026   Valor R$ 450,00",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let got = boleto::find_in_text(&rows, today).unwrap();
    assert_eq!(got.barcode, b.barcode);
    assert_eq!(got.amount, Some(45_000));
    assert_eq!(
        got.beneficiary.as_deref(),
        Some("Escola Exemplo de Testes Ltda")
    );
    // Collection: the due date comes from the text.
    let c = boleto::from_barcode(
        &collection_barcode(2, 9_912, "00001234567890123456789012345"),
        today,
    )
    .unwrap();
    let rows: Vec<String> = vec![
        "SANEAMENTO EXEMPLO".into(),
        "Data de vencimento: 05/11/2026".to_string(),
        c.line.clone(),
    ];
    let got = boleto::find_in_text(&rows, today).unwrap();
    assert_eq!(got.due_date.as_deref(), Some("2026-11-05"));
    assert_eq!(got.segment, Some(2));
    assert!(boleto::find_in_text(&["Nada de boleto aqui 123".into()], today).is_none());
}

#[test]
fn pix_copy_and_paste_with_crc() {
    let key = "123e4567-e89b-12d3-a456-426614174000";
    let p = super::br_code::payload(key, "Loja Exemplo", "Cidade Teste").unwrap();
    let c = pix::parse(&p).unwrap();
    assert_eq!(c.key.as_deref(), Some(key));
    assert_eq!(c.name.as_deref(), Some("LOJA EXEMPLO"));
    assert_eq!(c.city.as_deref(), Some("CIDADE TESTE"));
    assert_eq!(c.amount, None);
    assert!(!c.dynamic);
    // Wrong CRC: not Pix.
    let bad = format!("{}0000", &p[..p.len() - 4]);
    assert!(pix::parse(&bad).is_none());
    // With amount and URL (charge): builds the code with the right CRC.
    let tlv = |id: &str, v: &str| format!("{id}{:02}{v}", v.len());
    let acc = tlv("00", "br.gov.bcb.pix") + &tlv("25", "pix.exemplo.invalid/qr/v2/abc");
    let mut body = tlv("00", "01")
        + &tlv("01", "12")
        + &tlv("26", &acc)
        + &tlv("52", "0000")
        + &tlv("53", "986")
        + &tlv("54", "123.40")
        + &tlv("58", "BR")
        + &tlv("59", "Condominio Exemplo")
        + &tlv("60", "Cidade")
        + &tlv("62", &tlv("05", "TX123"));
    body += "6304";
    let crc = super::br_code::crc16(body.as_bytes());
    let code = format!("{body}{crc:04X}");
    let c = pix::parse(&code).unwrap();
    assert_eq!(c.amount, Some(12_340));
    assert!(c.dynamic);
    assert_eq!(c.txid.as_deref(), Some("TX123"));
    // Printed on paper, on two lines.
    let (a, b) = code.split_at(40);
    let rows = vec!["Pague com Pix".to_string(), a.to_string(), b.to_string()];
    assert_eq!(pix::find_in_text(&rows).unwrap().payload, code);
}

fn rows(s: &str) -> Vec<String> {
    s.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

#[test]
fn reads_a_pix_proof() {
    let text = "
        Comprovante de transferência
        Pix enviado
        Valor
        R$ 1.234,50
        Data 22/10/2026 - 14:05
        Destino
        Nome  FULANO DE TESTE SILVA
        CPF  ***.123.456-**
        Instituição  BANCO EXEMPLO S.A.
        Origem
        Nome  CICLANA DE TESTE
        ID da transação
        E18236120202610221705aBcDeFgHiJk
    ";
    let p = proof::read(&rows(text), None);
    assert_eq!(p.kind, ProofKind::Pix);
    assert_eq!(p.direction, Direction::Out);
    assert_eq!(p.amount, Some(123_450));
    assert_eq!(p.date.as_deref(), Some("2026-10-22"));
    assert_eq!(p.time.as_deref(), Some("14:05"));
    assert_eq!(p.counterpart.as_deref(), Some("FULANO DE TESTE SILVA"));
    assert_eq!(p.institution.as_deref(), Some("BANCO EXEMPLO S.A."));
    assert_eq!(
        p.e2e_id.as_deref(),
        Some("E18236120202610221705aBcDeFgHiJk")
    );
    assert_eq!(p.origin_bank.as_deref(), Some("Nubank"));
    assert!(p.checked);
    // The masked CPF is not kept anywhere.
    assert!(!format!("{p:?}").contains("123.456"));
}

#[test]
fn reads_a_received_pix_and_a_ted_with_label_on_the_same_line() {
    let p = proof::read(
        &rows(
            "Pix recebido\nValor recebido R$ 80,00\n23 out 2026 09:12\nQuem pagou: Maria Exemplo\n",
        ),
        None,
    );
    assert_eq!(p.direction, Direction::In);
    assert_eq!(p.amount, Some(8_000));
    assert_eq!(p.date.as_deref(), Some("2026-10-23"));
    assert_eq!(p.counterpart.as_deref(), Some("Maria Exemplo"));
    // Without an end-to-end ID, the Pix is left for review.
    assert!(!p.checked);
    let t = proof::read(
        &rows(
            "Comprovante de TED\nFavorecido: Empresa Teste Ltda\nValor: R$ 2.000,00\nData: 21/10/2026",
        ),
        None,
    );
    assert_eq!(t.kind, ProofKind::Transfer);
    assert_eq!(t.counterpart.as_deref(), Some("Empresa Teste Ltda"));
    assert_eq!(t.amount, Some(200_000));
    assert!(t.checked);
}

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
            json!({"name": "Conta Teste", "kind": "checking", "opening_date": "2026-01-01"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    (dir, api, acc)
}

#[test]
fn bill_becomes_a_planned_bill_once_with_reminder() {
    let (_d, api, acc) = api();
    let today = crate::time::today_local();
    let due = today + Duration::days(3);
    let f = 1000 + (due - d("2025-02-22")).num_days() as u32;
    let code = bank_barcode("001", f, 21_990, FREE);
    // Read from the pasted code: due date, amount and the default category.
    let r = api.call("payments.read", json!({"codes": [code]})).unwrap();
    assert_eq!(r["kind"], "bill");
    assert_eq!(r["bill"]["amount"], 21_990);
    assert_eq!(r["bill"]["due_date"], fmt_date(due));
    assert!(r["category_id"].is_string());
    assert!(r["existing"].is_null());
    let id = api
        .call(
            "payments.plan",
            json!({"account_id": acc, "description": r["description"], "amount": 21_990,
                   "due_date": fmt_date(due), "category_id": r["category_id"], "code": code}),
        )
        .unwrap();
    let t = api.call("transactions.get", json!({"id": id})).unwrap();
    assert_eq!(t["status"], "planned");
    assert_eq!(t["amount"], -21_990);
    assert!(
        t["notes"]
            .as_str()
            .unwrap()
            .contains(&r["bill"]["line"].as_str().unwrap()[..10])
    );
    // Read again: it is already posted, and does not post twice.
    let again = api.call("payments.read", json!({"codes": [code]})).unwrap();
    assert_eq!(again["existing"]["txn_id"], id);
    assert!(
        api.call(
            "payments.plan",
            json!({"account_id": acc, "description": "x", "amount": 21_990,
                   "due_date": fmt_date(due), "code": code}),
        )
        .is_err()
    );
    // Enters the bills due (the "due_soon" reminder).
    let up = api
        .with_session(|s, _| crate::reports::upcoming(s.db.conn(), today, 7))
        .unwrap();
    assert!(up.iter().any(|u| u.ref_id == id.as_str().unwrap()));
}

#[test]
fn proof_links_to_the_statement_and_closes_the_planned_bill() {
    let (_d, api, acc) = api();
    let today = crate::time::today_local();
    let f = 1000 + (today - d("2025-02-22")).num_days() as u32;
    let code = bank_barcode("237", f, 10_000, FREE);
    let planned = api
        .call(
            "payments.plan",
            json!({"account_id": acc, "description": "Escola Exemplo", "amount": 10_000,
                   "due_date": fmt_date(today), "code": code}),
        )
        .unwrap();
    let paid = api
        .call(
            "transactions.create",
            json!({"account_id": acc, "date": fmt_date(today - Duration::days(1)),
                   "amount": -10_000, "description": "PAGAMENTO BOLETO"}),
        )
        .unwrap()["id"]
        .clone();
    let text = format!(
        "Comprovante de pagamento\nPagamento de boleto\nBeneficiário: Escola Exemplo\nValor pago R$ 100,00\nData do pagamento {}\nCódigo de barras {}",
        today.format("%d/%m/%Y"),
        code
    );
    let r = api.call("payments.read", json!({"text": text})).unwrap();
    assert_eq!(r["kind"], "proof");
    assert_eq!(r["proof"]["kind"], "bill");
    assert_eq!(r["proof"]["barcode"], code);
    assert_eq!(r["matches"][0]["txn_id"], paid);
    assert_eq!(r["planned"][0]["txn_id"], planned);
    // Links to the statement transaction with the data read from the receipt (without keeping the
    // image) and removes the planned one.
    let id = api
        .call(
            "payments.save_proof",
            json!({"txn_id": paid, "date": fmt_date(today), "payee": "Escola Exemplo",
                   "close_planned_id": planned, "proof": r["proof"], "sha256": "ab".repeat(32)}),
        )
        .unwrap();
    assert_eq!(id, paid);
    let t = api.call("transactions.get", json!({"id": paid})).unwrap();
    assert_eq!(t["payee"], "Escola Exemplo");
    let notes = t["notes"].as_str().unwrap();
    assert!(notes.contains("Comprovante de boleto pago"), "{notes}");
    assert!(notes.contains(&file_mark(&"ab".repeat(32))));
    let atts = api
        .call("transactions.list_attachments", json!({"txn_id": paid}))
        .unwrap();
    assert!(atts.as_array().unwrap().is_empty());
    assert!(
        api.call("transactions.get", json!({"id": planned}))
            .is_err()
    );
}

#[test]
fn proof_without_statement_turns_the_planned_into_the_payment_or_creates_one() {
    let (_d, api, acc) = api();
    let today = crate::time::today_local();
    let f = 1000 + (today - d("2025-02-22")).num_days() as u32;
    let code = bank_barcode("104", f, 5_000, FREE);
    let planned = api
        .call(
            "payments.plan",
            json!({"account_id": acc, "description": "Conta", "amount": 5_000,
                   "due_date": fmt_date(today + Duration::days(2)), "code": code}),
        )
        .unwrap();
    api.call(
        "payments.save_proof",
        json!({"txn_id": planned, "date": fmt_date(today)}),
    )
    .unwrap();
    let t = api
        .call("transactions.get", json!({"id": planned}))
        .unwrap();
    assert_eq!(t["status"], "cleared");
    assert_eq!(t["date"], fmt_date(today));
    // Nothing in the statement: creates the transaction.
    let id = api
        .call(
            "payments.save_proof",
            json!({"create": {"account_id": acc, "amount": -3_000, "description": "Pix Fulano"},
                   "date": fmt_date(today), "payee": "Fulano de Teste"}),
        )
        .unwrap();
    let t = api.call("transactions.get", json!({"id": id})).unwrap();
    assert_eq!(t["amount"], -3_000);
    assert_eq!(t["payee"], "Fulano de Teste");
    assert!(
        api.call("payments.save_proof", json!({"date": fmt_date(today)}))
            .is_err()
    );
}

#[test]
fn image_attachment_carries_ocr_text_for_engines_without_vision() {
    use crate::ai::attachments::{self, EngineCaps};
    use crate::ai::providers::{Block, Msg, Role};
    let mut a = attachments::prepare(
        "comprovante.png",
        None,
        crate::ai::attachments::tests::png(40, 40),
    )
    .unwrap();
    assert!(a.needs_native());
    // Vision lines (as the app sends on the Mac).
    let lines = vec![
        crate::receipts::ocr::OcrLine {
            text: "Comprovante Pix enviado".into(),
            conf: 0.95,
            x: 0.1,
            y: 0.8,
            w: 0.5,
            h: 0.03,
            ..Default::default()
        },
        crate::receipts::ocr::OcrLine {
            text: "Valor R$ 50,00".into(),
            conf: 0.85,
            x: 0.1,
            y: 0.6,
            w: 0.5,
            h: 0.03,
            ..Default::default()
        },
    ];
    attachments::add_ocr(&mut a, Some(&lines));
    assert!(!a.needs_native());
    let text_only = EngineCaps {
        images: false,
        pdf: false,
        effort: false,
    };
    assert!(a.readable_by(text_only));
    let msg = Msg {
        role: Role::User,
        blocks: vec![Block::Attachment(std::sync::Arc::new(a.clone()))],
        raw: None,
    };
    let out = attachments::resolve(std::slice::from_ref(&msg), text_only);
    let all: String = out[0]
        .blocks
        .iter()
        .filter_map(|b| match b {
            Block::Text(t) => Some(t.clone()),
            _ => None,
        })
        .collect();
    assert!(
        all.contains("on-device OCR (90% average confidence"),
        "{all}"
    );
    assert!(all.contains("payment or transfer receipt"));
    assert!(all.contains("Valor R$ 50,00"));
    // With vision: the image and the text.
    let vision = EngineCaps {
        images: true,
        ..text_only
    };
    let out = attachments::resolve(&[msg], vision);
    assert!(
        out[0]
            .blocks
            .iter()
            .any(|b| matches!(b, Block::Image { .. }))
    );
    assert!(
        out[0]
            .blocks
            .iter()
            .any(|b| matches!(b, Block::Text(t) if t.contains("Valor R$ 50,00")))
    );
    // The person's choice: text only, or image only.
    let with = |mode| Msg {
        role: Role::User,
        blocks: vec![Block::Attachment(std::sync::Arc::new(
            attachments::Attachment {
                image_mode: mode,
                ..a.clone()
            },
        ))],
        raw: None,
    };
    let out = attachments::resolve(&[with(attachments::ImageMode::Text)], vision);
    assert!(
        !out[0]
            .blocks
            .iter()
            .any(|b| matches!(b, Block::Image { .. }))
    );
    let out = attachments::resolve(&[with(attachments::ImageMode::Image)], vision);
    assert!(
        out[0]
            .blocks
            .iter()
            .any(|b| matches!(b, Block::Image { .. }))
    );
    assert!(
        !out[0]
            .blocks
            .iter()
            .any(|b| matches!(b, Block::Text(t) if t.contains("Valor R$ 50,00")))
    );
    // Without vision and "image only": does not read.
    assert!(
        !attachments::Attachment {
            image_mode: attachments::ImageMode::Image,
            ..a.clone()
        }
        .readable_by(text_only)
    );
}

#[test]
fn br_code_example_from_the_central_bank_manual() {
    // Static QR example from the Pix Initiation Standards Manual (CRC 1D3D).
    let c = pix::parse(
        "00020126580014br.gov.bcb.pix0136123e4567-e12b-12d1-a456-4266554400005204000053039865802BR5913Fulano de Tal6008BRASILIA62070503***63041D3D",
    )
    .unwrap();
    assert_eq!(
        c.key.as_deref(),
        Some("123e4567-e12b-12d1-a456-426655440000")
    );
    assert_eq!(c.name.as_deref(), Some("Fulano de Tal"));
    assert_eq!(c.city.as_deref(), Some("BRASILIA"));
    assert_eq!(c.txid.as_deref(), Some("***"));
    assert_eq!(c.amount, None);
    // FEBRABAN factor for 23/10/2026: 1608.
    assert_eq!(
        boleto::due_from_factor(1608, d("2026-10-20")),
        Some(d("2026-10-23"))
    );
}

#[test]
fn scheduled_pix_is_not_a_payment_yet() {
    let p = proof::read(
        &rows(
            "Comprovante de agendamento\nPix agendado para 30/10/2026\nValor R$ 120,00\nPara: Fulano Exemplo",
        ),
        None,
    );
    assert!(p.scheduled);
    assert_eq!(p.amount, Some(12_000));
}

/// Test bench with the synthetic samples from the private folder (outside the repository):
/// `TREASURY_SAMPLES=~/Treasury-privado/documentos-amostras cargo test -p treasury-core
/// --lib payments::tests::bench -- --ignored --nocapture`. Numbers only in the output.
#[test]
#[ignore]
#[cfg(feature = "ocr-paddle")]
fn bench_with_private_samples() {
    let Ok(root) = std::env::var("TREASURY_SAMPLES") else {
        return;
    };
    let today = d("2026-10-23");
    let mut stats: std::collections::BTreeMap<String, (u32, u32)> = Default::default();
    for dir in [
        "boleto",
        "conta-consumo",
        "comprovante",
        "carne",
        "iptu",
        "pix-qr",
    ] {
        let Ok(list) = std::fs::read_dir(std::path::Path::new(&root).join(dir)) else {
            continue;
        };
        let mut files: Vec<_> = list.flatten().map(|e| e.path()).collect();
        files.sort();
        for g in files
            .iter()
            .filter(|p| p.to_string_lossy().ends_with(".gabarito.json"))
        {
            let want: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(g).unwrap()).unwrap();
            for f in want["arquivos"].as_array().unwrap() {
                let name = f.as_str().unwrap();
                if name.ends_with(".pdf") {
                    continue;
                }
                let bytes = std::fs::read(g.with_file_name(name)).unwrap();
                let Ok(o) = crate::ocr::read_text(None, 1.0, Some(&bytes)) else {
                    continue;
                };
                let (kind, bill, pix, proof) = classify(&o.rows, &[], today);
                let tipo = want["tipo"].as_str().unwrap_or("");
                let ok = if tipo.starts_with("comprovante") {
                    let v = want["valor"]
                        .as_str()
                        .map(|v| v.replace(['.', ','], "").parse::<i64>().unwrap());
                    kind == PaymentDocKind::Proof
                        && proof.as_ref().and_then(|p| p.amount) == v
                        && proof.as_ref().and_then(|p| p.counterpart.as_deref())
                            == want["recebedor"].as_str()
                        && proof.as_ref().and_then(|p| p.date.as_deref())
                            == want["data_hora"].as_str().and_then(|d| d.get(..10))
                } else if let Some(code) = want["codigo_barras"].as_str() {
                    bill.as_ref().map(|b| b.barcode.as_str()) == Some(code)
                } else if let Some(code) = want["copia_e_cola"].as_str() {
                    pix.as_ref().map(|p| p.payload.as_str()) == Some(code)
                } else {
                    continue;
                };
                if std::env::var("TREASURY_SAMPLES_DEBUG").is_ok() {
                    eprintln!("== {name} {kind:?} ok={ok}");
                    if let Some(p) = &proof {
                        eprintln!(
                            "   quem={:?} esperado={:?} data={:?} {:?} e2e={} banco={:?}",
                            p.counterpart,
                            want["recebedor"],
                            p.date,
                            want["data_hora"],
                            p.e2e_id.is_some(),
                            p.institution
                        );
                    }
                    if !ok || std::env::var("TREASURY_SAMPLES_DEBUG").as_deref() == Ok("2") {
                        for r in o.rows.iter().take(40) {
                            eprintln!("   | {r}");
                        }
                    }
                }
                let key = format!(
                    "{tipo}{}",
                    if name.contains("_foto") {
                        " (foto)"
                    } else {
                        ""
                    }
                );
                let e = stats.entry(key).or_default();
                e.1 += 1;
                if ok {
                    e.0 += 1;
                }
            }
        }
    }
    for (k, (ok, n)) in &stats {
        eprintln!("{k}: {ok}/{n}");
    }
}
