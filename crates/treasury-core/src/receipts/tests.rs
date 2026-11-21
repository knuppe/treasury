use serde_json::json;

use super::nfce;
use crate::api::Api;
use crate::paths::TreasuryPaths;

/// Página da consulta pública da NFC-e no leiaute das SEFAZ (como a de PR e MG), encurtada.
pub(crate) const PAGE: &str = r#"<html><body><div id="conteudo">
<div class="txtCenter"><div id="u20" class="txtTopo">SUPERMERCADO EXEMPLO LTDA</div>
<div class="text">CNPJ: 12.345.678/0001-90</div><div class="text">RUA DAS FLORES, 100, CENTRO, CURITIBA, PR</div></div>
<table id="tabResult" cellspacing="0" cellpadding="0">
<tr id="Item + 1"><td valign="top"><span class="txtTit">ARROZ TIO JOAO T1 5KG</span><span class="RCod">(Código: 7893500018346 )</span><br />
<span class="Rqtd"><strong>Qtde.:</strong>1</span><span class="RUN"><strong>UN: </strong>UN</span><span class="RvlUnit"><strong>Vl. Unit.:</strong>&nbsp;24,90</span></td>
<td align="right" valign="top" class="txtTit noWrap">Vl. Total<br /><span class="valor">24,90</span></td></tr>
<tr id="Item + 2"><td valign="top"><span class="txtTit">BANANA PRATA KG</span><span class="RCod">(Código: 2000001 )</span><br />
<span class="Rqtd"><strong>Qtde.:</strong>1,235</span><span class="RUN"><strong>UN: </strong>KG</span><span class="RvlUnit"><strong>Vl. Unit.:</strong>&nbsp;5,99</span></td>
<td align="right" valign="top" class="txtTit noWrap">Vl. Total<br /><span class="valor">7,40</span></td></tr>
<tr id="Item + 3"><td valign="top"><span class="txtTit">DETERG YPE NEUTRO 500ML</span><span class="RCod">(Código: 7896098900208 )</span><br />
<span class="Rqtd"><strong>Qtde.:</strong>2</span><span class="RUN"><strong>UN: </strong>UN</span><span class="RvlUnit"><strong>Vl. Unit.:</strong>&nbsp;2,79</span></td>
<td align="right" valign="top" class="txtTit noWrap">Vl. Total<br /><span class="valor">5,58</span></td></tr>
<tr id="Item + 4"><td valign="top"><span class="txtTit">CERV BRAHMA LT 350ML</span><span class="RCod">(Código: 7891149103102 )</span><br />
<span class="Rqtd"><strong>Qtde.:</strong>6</span><span class="RUN"><strong>UN: </strong>UN</span><span class="RvlUnit"><strong>Vl. Unit.:</strong>&nbsp;3,49</span></td>
<td align="right" valign="top" class="txtTit noWrap">Vl. Total<br /><span class="valor">20,94</span></td></tr>
</table>
<div id="totalNota" class="txtRight">
<div id="linhaTotal"><label>Qtd. total de itens:</label><span class="totalNumb">4</span></div>
<div id="linhaTotal"><label>Valor total R$:</label><span class="totalNumb">58,82</span></div>
<div id="linhaTotal"><label>Descontos R$:</label><span class="totalNumb">1,82</span></div>
<div id="linhaTotal" class="linhaShade"><label>Valor a pagar R$:</label><span class="totalNumb txtMax">57,00</span></div>
<div id="linhaForma"><label>Forma de pagamento:</label><span class="totalNumb txtTitR">Valor pago R$:</span></div>
<div id="linhaTotal"><label class="tx">Cartão de Débito</label><span class="totalNumb">57,00</span></div>
<div id="linhaTotal"><label class="txtObs">Informação dos Tributos Totais Incidentes (Lei Federal 12.741/2012)</label><span class="totalNumb txtObs">15,30</span></div>
</div>
<div id="infos"><ul><li><strong>Número: </strong>12345<strong> Série: </strong>1<strong> Emissão: </strong>19/10/2026 10:21:33 - Via Consumidor</li>
<li><strong>Protocolo de Autorização: </strong>141260000000000</li></ul>
<strong>Chave de acesso:</strong><br /><span class="chave">4126 1012 3456 7800 0190 6500 1000 0123 4510 0012 3452</span>
<strong>CONSUMIDOR - CPF:</strong> 123.456.789-00</div></div></body></html>"#;

/// O texto do PDF do DANFE NFC-e.
const DANFE: &str = "\
MERCADO DO BAIRRO LTDA
CNPJ: 98.765.432/0001-10 Rua A, 1 - Belo Horizonte - MG
DANFE NFC-e - Documento Auxiliar da Nota Fiscal de Consumidor Eletrônica
Código Descrição Qtde Un Vl Unit Vl Total
1 7891000100103 LEITE INTEGRAL ITALAC 1L 2 UN 4,79 9,58
2 7896004000014 PAO DE FORMA PULLMAN 1 UN 9,90 9,90
3 7891000055120 CAFE PILAO 500G 1 UN 21,90 21,90
Qtde. total de itens 3
Valor total R$ 41,38
Valor a Pagar R$ 41,38
Emissão: 18/10/2026 18:02:11
";

#[test]
fn reads_the_sefaz_page() {
    let key = nfce::access_key(
        "https://www.fazenda.pr.gov.br/nfce/qrcode?p=41261012345678000190650010000123451000123452|2|1|1|ABC",
    );
    assert_eq!(
        key.as_deref(),
        Some("41261012345678000190650010000123451000123452")
    );
    assert_eq!(nfce::uf_of_key(key.as_deref().unwrap()), Some("PR"));
    // Dígito verificador errado: não é chave.
    assert!(nfce::access_key("41261012345678000190650010000123451000123459").is_none());

    let p = nfce::parse_html(PAGE);
    assert_eq!(p.store_name.as_deref(), Some("SUPERMERCADO EXEMPLO LTDA"));
    assert_eq!(p.store_cnpj.as_deref(), Some("12345678000190"));
    assert_eq!(p.uf.as_deref(), Some("PR"));
    assert_eq!(p.date.as_deref(), Some("2026-10-19"));
    assert_eq!(p.issued_at.as_deref(), Some("2026-10-19T10:21:33"));
    assert_eq!(p.items.len(), 4, "{:?}", p.items);
    assert_eq!(p.items[1].description, "BANANA PRATA KG");
    assert_eq!(p.items[1].quantity.as_deref(), Some("1,235"));
    assert_eq!(p.items[1].unit.as_deref(), Some("KG"));
    assert_eq!(p.items[1].unit_price, Some(599));
    assert_eq!(p.items[1].total, 740);
    assert_eq!(p.items[0].code.as_deref(), Some("7893500018346"));
    assert_eq!((p.items_total, p.discount, p.paid), (5_882, 182, 5_700));
    assert_eq!(p.tax_total, Some(1_530));
    assert!(p.warnings.is_empty(), "{:?}", p.warnings);
    // O CPF do consumidor não vira nada.
    assert!(
        serde_json::to_string(&p)
            .unwrap()
            .find("123.456.789")
            .is_none()
    );
}

#[test]
fn reads_the_danfe_pdf_text() {
    let p = nfce::parse_text(DANFE);
    assert_eq!(p.store_name.as_deref(), Some("MERCADO DO BAIRRO LTDA"));
    assert_eq!(p.items.len(), 3, "{:?}", p.items);
    assert_eq!(p.items[0].description, "LEITE INTEGRAL ITALAC 1L");
    assert_eq!(p.items[0].quantity.as_deref(), Some("2"));
    assert_eq!(p.items[0].total, 958);
    assert_eq!((p.items_total, p.paid), (4_138, 4_138));
    assert_eq!(p.date.as_deref(), Some("2026-10-18"));
    assert_eq!(p.tax_total, None);
}

/// XML de uma NFC-e (como a loja manda por e-mail), encurtado.
pub(crate) const XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<nfeProc xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><NFe><infNFe Id="NFe35261012345678000199650010000012341000012342" versao="4.00">
<ide><cUF>35</cUF><mod>65</mod><dhEmi>2026-10-18T10:21:05-03:00</dhEmi></ide>
<emit><CNPJ>12345678000199</CNPJ><xNome>SUPERMERCADO EXEMPLO LTDA</xNome><xFant>MERCADO EXEMPLO</xFant></emit>
<dest><CPF>12345678909</CPF><xNome>CONSUMIDOR</xNome></dest>
<det nItem="1"><prod><cProd>7893500018346</cProd><xProd>ARROZ TIO JOAO T1 5KG</xProd><uCom>UN</uCom><qCom>1.0000</qCom><vUnCom>25.9000000000</vUnCom><vProd>25.90</vProd></prod><imposto><vTotTrib>3.10</vTotTrib></imposto></det>
<det nItem="2"><prod><cProd>2000</cProd><xProd>BANANA PRATA KG</xProd><uCom>KG</uCom><qCom>0.5350</qCom><vUnCom>5.99</vUnCom><vProd>3.20</vProd></prod><imposto><vTotTrib>0.00</vTotTrib></imposto></det>
<total><ICMSTot><vProd>29.10</vProd><vDesc>0.10</vDesc><vNF>29.00</vNF><vTotTrib>3.10</vTotTrib></ICMSTot></total>
<infAdic><infCpl>Trib aprox R$: 2,00 Federal, 1,00 Estadual, 0,10 Municipal</infCpl></infAdic>
</infNFe></NFe><protNFe><infProt><chNFe>35261012345678000199650010000012341000012342</chNFe></infProt></protNFe></nfeProc>"#;

#[test]
fn reads_the_nfe_xml() {
    assert!(nfce::looks_like_xml(XML));
    let p = nfce::parse_xml(XML);
    assert_eq!(
        p.access_key.as_deref(),
        Some("35261012345678000199650010000012341000012342")
    );
    assert_eq!(p.uf.as_deref(), Some("SP"));
    assert_eq!(p.store_name.as_deref(), Some("MERCADO EXEMPLO"));
    assert_eq!(p.store_cnpj.as_deref(), Some("12345678000199"));
    assert_eq!(p.date.as_deref(), Some("2026-10-18"));
    assert_eq!(p.issued_at.as_deref(), Some("2026-10-18T10:21:05"));
    assert_eq!(p.items.len(), 2);
    assert_eq!(p.items[1].quantity.as_deref(), Some("0,535"));
    assert_eq!(p.items[1].unit_price, Some(599));
    assert_eq!((p.items_total, p.discount, p.paid), (2_910, 10, 2_900));
    assert_eq!(p.tax_total, Some(310));
    assert_eq!(
        (p.tax_federal, p.tax_state, p.tax_municipal),
        (Some(200), Some(100), Some(10))
    );
    assert!(p.warnings.is_empty(), "{:?}", p.warnings);
    // Os dois jeitos de escrever os tributos aproximados.
    for (line, want) in [
        (
            "Trib aprox R$ 4,10 (Fed 2,10 Est 1,90 Mun 0,10)",
            (210, 190, 10),
        ),
        (
            "Trib aprox R$ 2,00 Federal 1,00 Estadual 0,10 Municipal",
            (200, 100, 10),
        ),
    ] {
        let mut r = nfce::ParsedReceipt::default();
        nfce::split_taxes(line, &mut r);
        assert_eq!(
            (r.tax_federal, r.tax_state, r.tax_municipal),
            (Some(want.0), Some(want.1), Some(want.2)),
            "{line}"
        );
    }
}

fn api() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new(paths).unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana", "Bruno"]}),
    )
    .unwrap();
    (dir, api)
}

fn member(api: &Api, name: &str) -> String {
    api.call("household.list_members", json!({}))
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == name)
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn imports_links_and_summarizes_what_each_member_buys() {
    let (_d, api) = api();
    let ana = member(&api, "Ana");
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Cartão da Ana", "kind": "checking", "member_ids": [ana], "opening_date": "2026-01-01"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let txn = api
        .call(
            "transactions.create",
            json!({"account_id": acc, "date": "2026-10-19", "amount": -5_700, "description": "SUPERMERCADO EXEMPLO"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Prévia pelo texto da página: grupos, imposto impresso rateado e o lançamento.
    let p = api.call("receipts.preview", json!({"text": PAGE})).unwrap();
    assert_eq!(p["tax"], 1_530);
    assert_eq!(p["tax_printed"], true);
    assert_eq!(p["txn"]["txn_id"], txn.as_str());
    let items = p["items"].as_array().unwrap();
    let groups: Vec<&str> = items.iter().map(|i| i["group"].as_str().unwrap()).collect();
    assert_eq!(
        groups,
        ["mercearia", "hortifruti", "limpeza", "bebidas_alcoolicas"]
    );
    let sum: i64 = items.iter().map(|i| i["tax"].as_i64().unwrap()).sum();
    assert_eq!(sum, 1_530, "o rateio fecha com a nota");
    assert!(items.iter().all(|i| i["tax_origin"] == "nfce"));
    // A cerveja leva mais imposto que o arroz.
    assert!(items[3]["tax_rate"].as_f64() > items[0]["tax_rate"].as_f64());

    let r = api.call("receipts.import", json!({"text": PAGE})).unwrap();
    assert_eq!(r["member_id"], ana.as_str(), "o membro vem do lançamento");
    assert_eq!(r["txn_id"], txn.as_str());
    assert_eq!(r["item_count"], 4);
    // A mesma nota de novo: só substituindo.
    assert!(api.call("receipts.import", json!({"text": PAGE})).is_err());
    let again = api
        .call("receipts.import", json!({"text": PAGE, "replace": true}))
        .unwrap();
    assert_ne!(again["id"], r["id"]);

    // Cupom do PDF, sem tributos impressos: imposto estimado pelo percentual do grupo, da família.
    let r2 = api.call("receipts.import", json!({"text": DANFE})).unwrap();
    assert_eq!(r2["tax_printed"], false);
    assert!(r2["tax"].as_i64().unwrap() > 0);
    assert!(r2["member_id"].is_null());

    // Foto lida pelo agente: itens direto.
    let r3 = api
        .call(
            "receipts.import",
            json!({"manual": {"date": "2026-10-10", "store_name": "Feira", "items": [
                {"description": "TOMATE KG", "total": 800},
                {"description": "ALFACE", "total": 350}
            ]}}),
        )
        .unwrap();
    assert_eq!(r3["origin"], "agent");

    // A prévia já traz a categoria de produto prevista de cada item.
    assert!(items.iter().all(|i| i["category_id"].is_string()));
    assert_eq!(items[0]["category_reason"], "words");

    // Troca da categoria à mão: vale para o produto, e o grupo de imposto acompanha.
    let id = again["id"].as_str().unwrap();
    let cats = api.call("food.categories", json!({})).unwrap();
    let higiene = cats
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["seed_key"] == "higiene")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let changed = api
        .call(
            "receipts.update",
            json!({"id": id, "categories": [{"line": 3, "category_id": higiene}]}),
        )
        .unwrap();
    assert_eq!(changed["items"][2]["group"], "higiene");
    assert_eq!(changed["items"][2]["group_origin"], "manual");
    assert_eq!(changed["items"][2]["category_id"], higiene.as_str());
    // O que o documento diz não muda.
    assert_eq!(changed["paid"], again["paid"]);
    // A mesma descrição, na próxima compra, já vem com a categoria escolhida.
    let g = api
        .call(
            "food.guess",
            json!({"description": changed["items"][2]["description"]}),
        )
        .unwrap();
    assert_eq!(g["reason"], "known");
    assert_eq!(g["category_id"], higiene.as_str());

    let s = api
        .call(
            "receipts.summary",
            json!({"from": "2026-10-01", "to": "2026-10-31"}),
        )
        .unwrap();
    assert_eq!(s["receipts"], 3);
    let members = s["members"].as_array().unwrap();
    let ana_part = members.iter().find(|m| m["member_name"] == "Ana").unwrap();
    // Por segmento de consumo (a raiz da categoria de produto), não mais pelo grupo do cupom.
    let segments = ana_part["groups"].as_array().unwrap();
    assert!(segments.iter().any(|g| g["label"] == "Alimentação"));
    assert!(segments.iter().all(|g| g["segment_id"].is_string()));
    assert!(members.iter().any(|m| m["member_name"] == "Família"));
    assert_eq!(s["tax_printed"], 1_530);
    assert!(s["top_items"].as_array().unwrap().len() >= 5);
    let list = api
        .call("receipts.list", json!({"from": "2026-10-01"}))
        .unwrap();
    assert_eq!(list.as_array().unwrap().len(), 3);

    // O imposto impresso entra na estimativa no lugar da média da categoria do lançamento.
    let e = api
        .call("tax_burden.estimate", json!({"year": 2026}))
        .unwrap();
    let lines = e["consumption"].as_array().unwrap();
    let nfce = lines
        .iter()
        .find(|l| l["group_key"] == "receipts")
        .expect("linha dos cupons");
    assert_eq!(nfce["spent"], 5_700);
    assert_eq!(nfce["taxes"]["probable"], 1_530);

    // Dias de trabalho para pagar os impostos, com a referência do Tesouro.
    assert!(e["work_days"].is_null(), "sem renda, sem dias");
    assert_eq!(e["work_days_reference"]["days"], 118);
    api.call(
        "transactions.create",
        json!({"account_id": acc, "date": "2026-10-05", "amount": 500_000, "description": "SALARIO"}),
    )
    .unwrap();
    let e = api
        .call("tax_burden.estimate", json!({"year": 2026}))
        .unwrap();
    let days = e["work_days"]["probable"].as_f64().unwrap();
    let share = e["share_of_income"]["probable"].as_f64().unwrap();
    assert!(
        (days - share * 365.0).abs() < 0.1 && days > 0.0,
        "{days} {share}"
    );

    api.call("receipts.delete", json!({"id": id})).unwrap();
    assert!(api.call("receipts.get", json!({"id": id})).is_err());
}

#[test]
fn outside_brazil_there_are_no_receipts() {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new(paths).unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"], "country": "PT"}),
    )
    .unwrap();
    let e = api
        .call("receipts.preview", json!({"text": PAGE}))
        .unwrap_err();
    assert!(e.to_string().contains("Brasil"), "{e}");
    // A categoria prevista sai em qualquer país (sem o grupo do cupom na interface).
    let g = api
        .call("food.guess", json!({"description": "XYZ 123"}))
        .unwrap();
    assert_eq!(g["reason"], "none");
}

/// O cupom fiscal aceita as três formas de pagamento da compra digitada: criar a despesa
/// junto (apagar o cupom apaga ela), ligar a um lançamento ou ficar sem lançamento.
#[test]
fn fiscal_receipt_takes_the_same_payment_choices_as_a_typed_purchase() {
    let (_d, api) = api();
    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Conta", "kind": "checking", "opening_date": "2026-01-01"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let txns = |api: &Api| {
        api.call("transactions.query_ids", json!({"account_ids": [acc]}))
            .unwrap()
            .as_array()
            .map(Vec::len)
            .unwrap_or(0)
    };
    // Criar junto: nasce a despesa com o valor pago, na categoria Mercado (quase tudo comida).
    let r = api
        .call(
            "receipts.import",
            json!({"text": PAGE, "payment": {"mode": "create", "account_id": acc}}),
        )
        .unwrap();
    assert_eq!(r["txn_created"], true);
    assert_eq!(txns(&api), 1);
    let id = r["id"].as_str().unwrap().to_string();
    // Passar para "sem lançamento" apaga a despesa criada.
    let r = api
        .call(
            "receipts.update",
            json!({"id": id, "payment": {"mode": "none"}}),
        )
        .unwrap();
    assert!(r["txn_id"].is_null());
    assert_eq!(txns(&api), 0);
    // Ligar a um lançamento com outro valor é recusado; com o mesmo, liga.
    let other = api
        .call(
            "transactions.create",
            json!({"account_id": acc, "date": "2026-10-19", "amount": -1_000, "description": "OUTRA"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        api.call(
            "receipts.update",
            json!({"id": id, "payment": {"mode": "link", "txn_id": other}}),
        )
        .is_err()
    );
    let same = api
        .call(
            "transactions.create",
            json!({"account_id": acc, "date": "2026-10-19", "amount": -5_700, "description": "MERCADO"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let r = api
        .call(
            "receipts.update",
            json!({"id": id, "payment": {"mode": "link", "txn_id": same}, "member_id": null}),
        )
        .unwrap();
    assert_eq!(r["txn_id"], same.as_str());
    assert_eq!(r["txn_created"], false);
    // Apagar o cupom não apaga o lançamento que já existia.
    api.call("receipts.delete", json!({"id": id})).unwrap();
    assert_eq!(txns(&api), 2);
    // Criado junto e apagado: a despesa sai com o cupom.
    let r = api
        .call(
            "receipts.import",
            json!({"text": DANFE, "payment": {"mode": "create", "account_id": acc}}),
        )
        .unwrap();
    assert_eq!(txns(&api), 3);
    api.call("receipts.delete", json!({"id": r["id"]})).unwrap();
    assert_eq!(txns(&api), 2);
}

#[test]
fn only_sefaz_links_are_fetched() {
    assert!(super::check_url("https://www.fazenda.pr.gov.br/nfce/qrcode?p=1").is_ok());
    assert_eq!(
        super::check_url("http://www.sefaz.rs.gov.br/NFCE/NFCE-COM.aspx?p=1").unwrap(),
        "https://www.sefaz.rs.gov.br/NFCE/NFCE-COM.aspx?p=1"
    );
    assert!(super::check_url("https://evil.example.com/?p=1").is_err());
    assert!(super::check_url("https://gov.br.evil.com/").is_err());
    assert!(super::check_url("file:///etc/passwd").is_err());
}

/// Grupos e avisos da leitura no idioma da instalação (os itens ficam como no cupom).
#[test]
fn groups_and_warnings_follow_the_installation_language() {
    use crate::i18n::{Locale, set_current};
    set_current(Locale::En);
    assert_eq!(super::ItemGroup::Hortifruti.label(), "Produce");
    assert_eq!(super::ItemGroup::Outros.label(), "Other");
    let p = nfce::parse_xml("<nfeProc></nfeProc>");
    assert_eq!(
        p.warnings,
        vec!["the XML doesn't have the invoice (infNFe)".to_string()]
    );
    set_current(Locale::PtBr);
    assert_eq!(super::ItemGroup::Hortifruti.label(), "Hortifrúti");
}

/// Gravar o cupom lido pelo link usa a página da prévia, sem consultar a SEFAZ de novo (que
/// pediria captcha); depois de gravado, o mesmo QR avisa que já foi importado, sem consultar.
#[test]
fn import_reuses_the_preview_page_and_never_queries_twice() {
    let (_d, api) = api();
    // Link de uma SEFAZ que não existe: qualquer consulta de verdade falharia.
    let key = "41261012345678000190650010000123451000123452";
    let url = format!("https://nfce.sefaz.exemplo.gov.br/consulta?p={key}|2|1|1|X");
    super::remember_page(key, PAGE);
    let p = api.call("receipts.preview", json!({ "url": url })).unwrap();
    assert_eq!(p["items"].as_array().unwrap().len(), 4);
    let r = api.call("receipts.import", json!({ "url": url })).unwrap();
    assert_eq!(r["store_name"], "SUPERMERCADO EXEMPLO LTDA");
    let e = api
        .call("receipts.preview", json!({ "url": url }))
        .unwrap_err();
    assert_eq!(e.code(), "conflict");
    assert!(e.message().contains("já foi importado"), "{}", e.message());
}
