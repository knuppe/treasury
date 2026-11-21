//! A resposta da IA e a data e a loja do cupom (dados inventados).

use super::*;

#[test]
fn ai_json_goes_through_the_same_checks() {
    let text = "Aqui está:\n```json\n{\"store_name\": \"Mercado Exemplo\", \"date\": \"2026-10-20\", \
        \"items\": [{\"description\": \"ARROZ 5KG\", \"quantity\": \"1\", \"unit_price\": 24.9, \"total\": 24.9}, \
        {\"description\": \"BANANA KG\", \"quantity\": 0.734, \"total\": 5.13}, \
        {\"description\": \"DESCONTO\", \"total\": -1.0}], \"total\": 29.03, \"item_count\": 2}\n```";
    let (r, m) = from_ai_json(text).unwrap();
    assert_eq!(r.items.len(), 2);
    assert_eq!(r.discount, 100);
    assert_eq!(r.total, Some(2903));
    assert_eq!(r.tier, Tier::Checked);
    assert_eq!(m.date, "2026-10-20");
    assert_eq!(m.store_name.as_deref(), Some("Mercado Exemplo"));
    assert_eq!(m.items[1].quantity.as_deref(), Some("0.734"));
    assert_eq!(m.paid, Some(2903));
    assert_eq!(m.from_photo, Some(true));
    // Conta que não fecha continua sem conferir.
    let (r, _) =
        from_ai_json(r#"{"items": [{"description": "PAO", "total": 5}], "total": 9}"#).unwrap();
    assert_eq!(r.tier, Tier::Unbalanced);
    assert!(from_ai_json("não consegui ler").is_none());
}

#[test]
fn date_and_store_from_the_rows() {
    let rows: Vec<String> = [
        "MERCADO DO BAIRRO",
        "CNPJ: 00.000.000/0001-00",
        "Emissão: 21/10/2026 18:13:33",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    assert_eq!(find_date(&rows, "BR").as_deref(), Some("2026-10-21"));
    assert_eq!(find_store(&rows).as_deref(), Some("MERCADO DO BAIRRO"));
    // EUA: mês antes do dia.
    let us = vec!["10/21/26 7:05 PM".to_string()];
    assert_eq!(find_date(&us, "US").as_deref(), Some("2026-10-21"));
    assert_eq!(find_date(&["sem data".to_string()], "BR"), None);
}
