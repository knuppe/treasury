//! Invented receipts, written as the OCR would return them (with its typical errors).

use super::*;

/// One box per line, from top to bottom.
fn ocr(rows: &[&str]) -> Vec<OcrLine> {
    let n = rows.len() as f32;
    rows.iter()
        .enumerate()
        .map(|(i, t)| OcrLine {
            text: t.to_string(),
            conf: 0.9,
            x: 0.05,
            y: 1.0 - (i as f32 + 1.0) / (n + 1.0),
            w: 0.9,
            h: 0.6 / (n + 1.0),
            tl: None,
            tr: None,
        })
        .collect()
}

#[test]
fn brazil_nfce_with_two_line_items_is_checked_by_item_count() {
    let r = read(
        &ocr(&[
            "MERCADO DO BAIRRO LTDA",
            "CNPJ: 00.000.000/0001-00",
            "DANFE NFC-e - Documento Auxiliar da Nota Fiscal de Consumidor Eletrônica",
            "001 7891234567895 ARROZ TIPO 1 5KG",
            "1 UN X 24,90  24,90",
            "002 2000123000000 BANANA PRATA KG",
            "0,734 KG X 6,99  5,13",
            "003 7890000000017 DETERGENTE 500ML  2,49G",
            "QTD. TOTAL DE ITENS  3",
            "VALOR TOTAL R$  32,52",
            "VALOR A PAGAR R$  32,52",
            "Cartão de Débito  32,52",
        ]),
        "BR",
        0.5,
    );
    let totals: Vec<i64> = r.items.iter().map(|i| i.total).collect();
    assert_eq!(totals, vec![2490, 513, 249]);
    assert_eq!(r.items[1].description, "BANANA PRATA KG");
    assert_eq!(r.items[1].quantity.as_deref(), Some("0.734"));
    assert_eq!(r.items[1].line_checked, Some(true));
    assert_eq!(r.total, Some(3252));
    assert_eq!(r.printed_count, Some(3));
    assert_eq!(r.tier, Tier::Checked);
}

#[test]
fn ocr_mistakes_in_values_are_fixed() {
    // Dot in place of the comma, "O" in place of the zero, tax-rate letter stuck on.
    let r = read(
        &ocr(&[
            "C 6% PAO DE FORMA  2.15",
            "E 23% SUMO LARANJA 1L  1,5O",
            "C 6% QUEIJO FLAMENGO  U,35",
            "Total  4,00",
        ]),
        "PT",
        0.5,
    );
    let totals: Vec<i64> = r.items.iter().map(|i| i.total).collect();
    assert_eq!(totals, vec![215, 150, 35]);
    assert_eq!(r.tier, Tier::Balanced);
}

#[test]
fn weight_line_after_item_and_total_after_savings() {
    let r = read(
        &ocr(&[
            "C 6% TOMATE C/ RAMA KG  0,54",
            "0,342 X 1,59",
            "C 6% * OVOS L 6UNID",
            "2 X 0,94  1,88",
            "Total  2,42",
            "Poupança Imediata  0,30",
            "Total  2,12",
        ]),
        "PT",
        0.5,
    );
    assert_eq!(r.items.len(), 2);
    assert_eq!(r.items[0].quantity.as_deref(), Some("0.342"));
    assert_eq!(r.items[1].description, "C 6% * OVOS L 6UNID");
    assert_eq!(r.discount, 30);
    assert_eq!(r.total, Some(212));
    assert_eq!(r.tier, Tier::Balanced);
}

#[test]
fn uk_receipt_with_tax_letters_and_card_payment() {
    let r = read(
        &ocr(&[
            "TESTCO STORES",
            "OAT BISCUITS  £1.50V",
            "MILK 2 PINT  £1.20D",
            "BANANAS",
            "0.605 kg @ 0.76/kg  £O.46D",
            "TOTAL  £3.16",
            "CARD  £3.16",
            "No. ITEMS SOLD 3",
        ]),
        "GB",
        0.5,
    );
    let totals: Vec<i64> = r.items.iter().map(|i| i.total).collect();
    assert_eq!(totals, vec![150, 120, 46]);
    assert_eq!(r.items[2].description, "BANANAS");
    assert_eq!(r.tier, Tier::Checked);
}

#[test]
fn argentina_quantity_line_before_the_item() {
    let r = read(
        &ocr(&[
            "SUPERMERCADO EJEMPLO S.A.",
            "2,000 x 1.250,00",
            "YERBA MATE 1KG (21.0)  2.500,00",
            "GALLETITAS X3 (21.0)  $ 1700.00",
            "TOTAL  $ 4.200,00",
        ]),
        "AR",
        0.5,
    );
    assert_eq!(r.items.len(), 2);
    assert_eq!(r.items[0].quantity.as_deref(), Some("2"));
    assert_eq!(r.items[0].line_checked, Some(true));
    assert_eq!(r.items[1].total, 170_000);
    assert_eq!(r.tier, Tier::Balanced);
}

#[test]
fn chile_pesos_without_cents() {
    let r = read(
        &ocr(&[
            "PAN MARRAQUETA  $1.990",
            "LECHE ENTERA 1L  $1.250",
            "TOTAL  $3.240",
        ]),
        "CL",
        0.5,
    );
    assert_eq!(r.total, Some(324_000));
    assert_eq!(r.tier, Tier::Balanced);
}

#[test]
fn missing_line_leaves_the_receipt_unbalanced() {
    let r = read(
        &ocr(&["ARROZ 5KG  24,90", "FEIJAO 1KG  8,49", "TOTAL R$  41,38"]),
        "BR",
        0.5,
    );
    assert_eq!(r.items_sum, 3339);
    assert_eq!(r.tier, Tier::Unbalanced);
}

#[test]
fn tilted_photo_rows_are_rebuilt() {
    // Tilted photo: the description and the value of the same line come out at different heights.
    let slope = 0.08_f32;
    let mut lines = vec![];
    for (i, (d, v)) in [
        ("CAFE 500G", "15,90"),
        ("ACUCAR 1KG", "4,99"),
        ("TOTAL", "20,89"),
    ]
    .iter()
    .enumerate()
    {
        let y = 0.8 - i as f32 * 0.1;
        let corners = |x0: f32, w: f32, y0: f32| {
            (Some([x0, y0 + 0.02]), Some([x0 + w, y0 + 0.02 + w * slope]))
        };
        let (tl, tr) = corners(0.05, 0.4, y);
        lines.push(OcrLine {
            text: d.to_string(),
            conf: 0.9,
            x: 0.05,
            y,
            w: 0.4,
            h: 0.02,
            tl,
            tr,
        });
        let vy = y + 0.55 * slope;
        let (tl, tr) = corners(0.7, 0.2, vy);
        lines.push(OcrLine {
            text: v.to_string(),
            conf: 0.9,
            x: 0.7,
            y: vy,
            w: 0.2,
            h: 0.02,
            tl,
            tr,
        });
    }
    let r = read(&lines, "BR", 1.0);
    assert_eq!(r.total, Some(2089));
    assert_eq!(r.tier, Tier::Balanced);
}

/// Private test bench (docs/ocr-receipts.md §12): `TREASURY_OCR_BENCH` points to the
/// output of an engine (`vision_ocr` JSON), with the answer key next to each photo. None of it
/// stays in the repository.
#[test]
#[ignore]
fn private_bench() {
    let Ok(path) = std::env::var("TREASURY_OCR_BENCH") else {
        return;
    };
    let data: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let money = |v: &serde_json::Value, cc: &str| -> Option<i64> {
        let s = v.as_str()?;
        let d = if cc == "CL" {
            to_decimal(s, "CL")
        } else if s.rfind(',') > s.rfind('.') {
            to_decimal(s, "BR")
        } else {
            to_decimal(s, "US")
        }?;
        Some(cents(d))
    };
    let (mut n, mut ok, mut bal, mut bal_ok, mut chk, mut chk_ok) = (0, 0, 0, 0, 0, 0);
    for rec in &data {
        let file = rec["file"].as_str().unwrap();
        let Ok(g) = std::fs::read_to_string(format!("{file}.gabarito.json")) else {
            continue;
        };
        let g: serde_json::Value = serde_json::from_str(&g).unwrap();
        if g["legivel"] != "total" || g["total"].is_null() {
            continue;
        }
        let cc = file.split("recibos-amostras/").nth(1).unwrap()[..2].to_string();
        let lines: Vec<OcrLine> = serde_json::from_value(rec["lines"].clone()).unwrap();
        let aspect = rec["width"].as_f64().unwrap() / rec["height"].as_f64().unwrap();
        let r = read(&lines, &cc, aspect as f32);
        let good = r.total.is_some() && r.total == money(&g["total"], &cc);
        n += 1;
        ok += good as u32;
        if r.tier >= Tier::Balanced {
            bal += 1;
            bal_ok += good as u32;
        }
        if r.tier == Tier::Checked {
            chk += 1;
            chk_ok += good as u32;
        }
    }
    println!(
        "n={n} total certo {ok} | fechou {bal} ({bal_ok} certos) | conferido {chk} ({chk_ok} certos)"
    );
}
